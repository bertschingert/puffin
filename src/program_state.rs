pub struct ProgramState<'a, T: std::io::Write> {
    /// Vector of values of variables
    vars: Vec<i64>,

    /// Where to write output to, typically stdout
    pub out: &'a mut T,
}

impl<'a, T: std::io::Write> ProgramState<'a, T> {
    pub fn new(num_vars: usize, out: &'a mut T) -> Self {
        ProgramState {
            vars: vec![0; num_vars],
            out,
        }
    }

    pub fn get_variable(&self, var: usize) -> i64 {
        self.vars[var]
    }

    pub fn set_variable(&mut self, var: usize, new_val: i64) {
        self.vars[var] = new_val;
    }
}
