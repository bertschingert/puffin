use std::sync::Mutex;

use crate::ast::{Expression, FileState};

pub struct ProgramState<'a, 'b, T: crate::SyncWrite> {
    vars: VariableState<'b>,

    /// Where to write output to, typically stdout
    pub out: &'a T,
}

// XXX: use more descriptive lifetime names for this...
impl<'a, 'b, T: crate::SyncWrite> ProgramState<'a, 'b, T> {
    pub fn new(num_vars: usize, out: &'a mut T) -> Self {
        ProgramState {
            vars: VariableState::new(num_vars),
            out,
        }
    }

    pub fn vars(&self) -> &VariableState {
        &self.vars
    }
}

/// Global variables can be accessed in two contexts: generally when evaluating a Variable the
/// mutex protecting them is not already held. VariableState::Locked represents this state.
///
/// When a vaiable is evaluated on the right-hand side of an assignment to another global variable,
/// then the variables are already unlocked, so VariableState::Unlocked will hold a reference to
/// the variables so that they can be evaluted without needing to acquire the already-acquired
/// mutex.
pub enum VariableState<'a> {
    Locked(LockedVars),
    Unlocked(UnlockedVars<'a>),
}

impl<'a> VariableState<'a> {
    fn new(num_vars: usize) -> Self {
        VariableState::Locked(LockedVars::new(num_vars))
    }

    pub fn get_variable(&self, var: usize) -> i64 {
        match self {
            VariableState::Locked(l) => l.get_variable(var),
            VariableState::Unlocked(u) => u.vars[var],
        }
    }

    pub fn set_variable_expression(
        &self,
        assignee: usize,
        f: Option<&FileState>,
        expr: &Expression,
    ) -> crate::Result<()> {
        match self {
            VariableState::Locked(l) => l.set_variable_expression(assignee, f, expr),
            VariableState::Unlocked(_) => panic!("Cannot assign to unlocked variable"),
        }
    }
}

pub struct UnlockedVars<'a> {
    vars: &'a Vec<i64>,
}

pub struct LockedVars {
    /// Vector of values of variables
    vars: Mutex<Vec<i64>>,
}

impl LockedVars {
    fn new(num_vars: usize) -> Self {
        LockedVars {
            vars: Mutex::new(vec![0; num_vars]),
        }
    }

    fn get_variable(&self, var: usize) -> i64 {
        let vars = self.vars.lock().unwrap();

        vars[var]
    }

    fn set_variable_expression(
        &self,
        assignee: usize,
        f: Option<&FileState>,
        expr: &Expression,
    ) -> crate::Result<()> {
        let mut vars = self.vars.lock().unwrap();
        let unlocked = UnlockedVars { vars: &*vars };
        let new = expr.evaluate(f, &VariableState::Unlocked(unlocked))?;
        vars[assignee] = new.to_integer();

        Ok(())
    }
}
