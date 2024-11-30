use std::collections::HashMap;
use std::sync::Mutex;

use crate::ast::{Expression, FileState};
use crate::types::{ArraySubscript, Value, Variable};

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

    /// Get the value of a variable `var`, within the context of the given `FileState`.
    ///
    /// This can fail if getting the value has to do filesystem I/O, for example, if an array
    /// subscript includes a file attribute.
    pub fn get_variable(&self, f: Option<&FileState>, var: &Variable) -> crate::Result<Value> {
        Ok(Value::Integer(match self {
            VariableState::Locked(l) => l.get_variable(f, self, var)?,
            VariableState::Unlocked(u) => u.get_variable(var),
        }))
    }

    pub fn set_variable_expression(
        &self,
        assignee: &Variable,
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

impl<'a> UnlockedVars<'a> {
    fn get_variable(&self, var: &Variable) -> i64 {
        match var {
            Variable::Id(id) => self.vars[id.id],
            Variable::Arr(_) => {
                todo!("Cannot yet evaluate an array value on the RHS of an assignment")
            }
        }
    }
}

pub struct LockedVars {
    /// Vector of values of variables
    vars: Mutex<Vec<i64>>,
    arrays: Mutex<Arrays>,
}

impl LockedVars {
    fn new(num_vars: usize) -> Self {
        LockedVars {
            vars: Mutex::new(vec![0; num_vars]),
            arrays: Mutex::new(Arrays::new(0)),
        }
    }

    fn get_variable(
        &self,
        f: Option<&FileState>,
        s: &VariableState,
        var: &Variable,
    ) -> crate::Result<i64> {
        Ok(match var {
            Variable::Id(id) => {
                let vars = self.vars.lock().unwrap();
                vars[id.id]
            }
            Variable::Arr(arr) => {
                let mut arrays = self.arrays.lock().unwrap();
                arrays.get_variable(f, s, arr)?
            }
        })
    }

    fn set_variable_expression(
        &self,
        assignee: &Variable,
        f: Option<&FileState>,
        expr: &Expression,
    ) -> crate::Result<()> {
        match assignee {
            Variable::Id(id) => {
                let mut vars = self.vars.lock().unwrap();
                let unlocked = UnlockedVars { vars: &*vars };
                let new = expr.evaluate(f, &VariableState::Unlocked(unlocked))?;
                vars[id.id] = new.to_integer();
            }
            Variable::Arr(_) => todo!(),
        };

        Ok(())
    }
}

struct Arrays {
    arrs: Vec<HashMap<Value, i64>>,
}

impl Arrays {
    fn new(num_arrays: usize) -> Self {
        Arrays {
            arrs: (0..num_arrays).map(|_| HashMap::new()).collect(),
        }
    }

    /// Gets a value from an associate array by evaluating the subscript and looking up the entry
    /// in the underlying hashmap for that value.
    ///
    /// If there is no entry in the map for that value, then the default result is 0.
    ///
    /// Fails if evaluating the subscript expression fails, which can occur if it has to do
    /// filesystem I/O.
    fn get_variable(
        &mut self,
        f: Option<&FileState>,
        s: &VariableState,
        arr: &ArraySubscript,
    ) -> crate::Result<i64> {
        Ok(*self.arrs[arr.id]
            .entry(arr.subscript.evaluate(f, s)?)
            .or_insert(0))
    }
}
