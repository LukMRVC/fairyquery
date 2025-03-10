pub struct InputHistory {
    history: Vec<String>,
    current: String,
    current_index: usize,
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current: String::new(),
            current_index: 0,
        }
    }

    pub fn push(&mut self) -> &mut String {
        self.current.clear();
        &mut self.current
    }

    pub fn get(&mut self) -> &str {
        // remove trailing newline character
        self.current = self.current.trim().to_string();
        self.history.push(self.current.clone());
        self.current_index = self.history.len();
        &self.current
    }
}
