use super::lang_trait::Language;

pub struct S10K {
    counter: usize,
}

impl S10K {
    pub const fn new() -> Self {
        Self { counter: 0 }
    }
}

const HELP: &str = r#"
S10K, aka S10000, the first test language for Try In Browser
Ignores the code and input, and prints 10,000 copies of "S".

Accepted arguments:
-h    Show this help and exit
"#;

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
    fn homepage(&self) -> &'static str {
        "https://bubbler-4.github.io/TryInBrowser/#S10K"
    }
    fn help(&self) -> &'static str {
        HELP.trim()
    }
}
