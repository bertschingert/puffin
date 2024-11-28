use std::sync::Mutex;

use crate::ast::{Expression, FileState};

pub struct ProgramState<'a, T: crate::SyncWrite> {
    /// Vector of values of variables
    vars: Mutex<Vec<i64>>,

    /// Where to write output to, typically stdout
    pub out: &'a T,
}

impl<'a, T: crate::SyncWrite> ProgramState<'a, T> {
    pub fn new(num_vars: usize, out: &'a mut T) -> Self {
        ProgramState {
            vars: Mutex::new(vec![0; num_vars]),
            out,
        }
    }

    pub fn get_variable(&self, var: usize) -> i64 {
        let vars = self.vars.lock().unwrap();

        vars[var]
    }

    pub fn set_variable_expression(
        &self,
        assignee: usize,
        f: Option<&FileState>,
        expr: &Expression,
    ) -> crate::Result<()> {
        let mut vars = self.vars.lock().unwrap();
        let new = expr.evaluate(f, self, Some(&vars))?;
        vars[assignee] = new.to_integer();

        Ok(())
    }
}
