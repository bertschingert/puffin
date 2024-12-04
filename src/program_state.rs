use crate::variables::*;
use crate::RuntimeError;

pub struct ProgramState<'a, 'b, T: crate::SyncWrite> {
    vars: VariableState<'b>,

    /// Where to write output to, typically stdout
    pub out: &'a T,

    /// Stores the first runtime error that occurs, so that workers can observe if another worker
    /// encountered a runtime error:
    runtime_error: std::sync::OnceLock<RuntimeError>,
}

// XXX: use more descriptive lifetime names for this...
impl<'a, 'b, T: crate::SyncWrite> ProgramState<'a, 'b, T> {
    pub fn new(num_scalars: usize, num_arrays: usize, out: &'a mut T) -> Self {
        ProgramState {
            vars: VariableState::new(num_scalars, num_arrays),
            out,
            runtime_error: std::sync::OnceLock::new(),
        }
    }

    pub fn vars(&self) -> &VariableState {
        &self.vars
    }

    /// Returns true if `runtime_error` has been set, indicating that some worker experienced an
    /// error.
    pub fn check_runtime_error(&self) -> bool {
        self.runtime_error.get().is_some()
    }

    /// Attempt to log a runtime error. It doesn't matter if it succeeds since we only care if at
    /// least one thread reports an error.
    pub fn set_runtime_error(&self, e: crate::RuntimeError) {
        let _ = self.runtime_error.set(e);
    }
}
