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
            VariableState::Unlocked(u) => u.get_variable(f, var)?,
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

#[derive(Clone)]
pub struct UnlockedVars<'a> {
    scalars: &'a Vec<i64>,
    arrays: &'a Arrays,
}

impl<'a> UnlockedVars<'a> {
    fn get_variable(&self, f: Option<&FileState>, var: &Variable) -> crate::Result<i64> {
        Ok(match var {
            Variable::Id(id) => self.scalars[id.id],
            Variable::Arr(arr) => {
                self.arrays
                    .get_variable(f, &VariableState::Unlocked(self.clone()), arr)?
            }
        })
    }
}

pub struct LockedVars {
    /// Vector of values of variables
    // XXX: only us a single mutex for both of these -- since we'll always be locking both anyways?
    scalars: Mutex<Vec<i64>>,
    arrays: Mutex<Arrays>,
}

impl LockedVars {
    fn new(num_vars: usize) -> Self {
        LockedVars {
            scalars: Mutex::new(vec![0; num_vars]),
            // TODO: proper size for var and arrays mutex
            arrays: Mutex::new(Arrays::new(num_vars)),
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
                let scalars = self.scalars.lock().unwrap();
                scalars[id.id]
            }
            Variable::Arr(arr) => {
                let arrays = self.arrays.lock().unwrap();
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
        let mut scalars = self.scalars.lock().unwrap();
        let mut arrays = self.arrays.lock().unwrap();
        let unlocked = UnlockedVars {
            scalars: &*scalars,
            arrays: &*arrays,
        };
        let new = expr.evaluate(f, &VariableState::Unlocked(unlocked))?;

        match assignee {
            Variable::Id(id) => {
                scalars[id.id] = new.to_integer();
            }
            Variable::Arr(arr) => {
                let unlocked = UnlockedVars {
                    scalars: &*scalars,
                    arrays: &*arrays,
                };
                let subscript = arr
                    .subscript
                    .evaluate(f, &VariableState::Unlocked(unlocked))?;
                arrays.set_variable(arr.id, subscript, new);
            }
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
        &self,
        f: Option<&FileState>,
        s: &VariableState,
        arr: &ArraySubscript,
    ) -> crate::Result<i64> {
        Ok(
            match self.arrs[arr.id].get(&arr.subscript.evaluate(f, s)?) {
                Some(v) => *v,
                _ => 0,
            },
        )
    }

    /// Sets a value in an associative array.
    fn set_variable(&mut self, id: usize, subscript: Value, new: Value) {
        self.arrs[id]
            .entry(subscript)
            .insert_entry(new.to_integer());
    }
}
