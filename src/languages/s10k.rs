use super::lang_trait::Language;

pub struct S10K {
    counter: usize,
}

impl S10K {
    pub const fn new() -> Self {
        Self { counter: 0 }
    }
}

impl Language for S10K {
    fn init(&mut self, _code: &str, _input: &str, _args: &str) {
        self.counter = 0;
    }
    fn step(&mut self) -> (String, String, bool) {
        for _ in 0..1000 {
            self.counter += 1;
        }
        ("S".to_string(), "".to_string(), self.counter < 100_000)
    }
}
