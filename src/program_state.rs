use std::collections::HashMap;

pub struct ProgramState {
    /// Vector of values of variables
    vars: Vec<u64>,

    /// Map from variable names (Strings) to index in variable vector
    var_map: HashMap<String, usize>,
}

impl ProgramState {
    pub fn new() -> Self {
        ProgramState {
            vars: Vec::new(),
            var_map: HashMap::new(),
        }
    }

    /// Get the index into the vars vector for new_var.
    ///
    /// If the name new_var is a pre-existing variable, then gets its existing
    /// index, otherwise, allocates a new index.
    pub fn add_variable(&mut self, new_var: &str) -> usize {
        *self.var_map.entry(new_var.to_string()).or_insert_with(|| {
            self.vars.push(0); // Variables are initialized to 0.
            self.vars.len() - 1
        })
    }

    pub fn get_variable(&self, var: usize) -> u64 {
        self.vars[var]
    }

    pub fn set_variable(&mut self, var: usize, new_val: u64) {
        self.vars[var] = new_val;
    }
}
