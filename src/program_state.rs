use crate::variables::*;

pub struct ProgramState<'a, 'b, T: crate::SyncWrite> {
    vars: VariableState<'b>,

    /// Where to write output to, typically stdout
    pub out: &'a T,
}

// XXX: use more descriptive lifetime names for this...
impl<'a, 'b, T: crate::SyncWrite> ProgramState<'a, 'b, T> {
    pub fn new(num_scalars: usize, num_arrays: usize, out: &'a mut T) -> Self {
        ProgramState {
            vars: VariableState::new(num_scalars, num_arrays),
            out,
        }
    }

    pub fn vars(&self) -> &VariableState {
        &self.vars
    }
}
