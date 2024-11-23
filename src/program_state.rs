pub struct ProgramState {
    /// Vector of values of variables
    vars: Vec<u64>,
}

impl ProgramState {
    pub fn new(num_vars: usize) -> Self {
        ProgramState {
            vars: vec![0; num_vars]
        }
    }

    pub fn get_variable(&self, var: usize) -> u64 {
        self.vars[var]
    }

    pub fn set_variable(&mut self, var: usize, new_val: u64) {
        self.vars[var] = new_val;
    }
}
