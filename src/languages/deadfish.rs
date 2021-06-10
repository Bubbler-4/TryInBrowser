use super::lang_trait::Language;

#[derive(PartialEq, Eq)]
enum OutputMode {
    Num,
    Char,
}

pub struct Deadfish {
    counter: usize,
    code: Vec<u8>,
    idx: usize,
    output_mode: OutputMode,
}

impl Deadfish {
    pub const fn new() -> Self {
        Self {
            counter: 0,
            code: vec![],
            idx: 0,
            output_mode: OutputMode::Num,
        }
    }
}

const HELP: &str = r#"
Deadfish (https://esolangs.org/wiki/Deadfish)
Accepted arguments:
-h    Show this help and exit
-o    Output as charcode
-n    Output as number (default)
"#;

impl Language for Deadfish {
    fn init(&mut self, code: &str, _input: &str, args: &str) {
        self.code.clear();
        self.code.extend_from_slice(code.as_bytes());
        self.idx = 0;
        self.output_mode = if args == "-o" {
            OutputMode::Char
        } else {
            OutputMode::Num
        };
        self.counter = 0;
    }
    fn step(&mut self) -> (String, String, bool) {
        if self.idx >= self.code.len() {
            ("".to_string(), "".to_string(), false)
        } else {
            let cmd = self.code[self.idx];
            self.idx += 1;
            match cmd {
                b'i' => {
                    self.counter = if self.counter == 255 {
                        0
                    } else {
                        self.counter + 1
                    }
                }
                b'd' => {
                    self.counter = if self.counter == 0 || self.counter == 257 {
                        0
                    } else {
                        self.counter - 1
                    }
                }
                b's' => {
                    self.counter = if self.counter == 16 {
                        0
                    } else {
                        self.counter.pow(2)
                    }
                }
                _ => (),
            };
            if cmd == b'o' {
                if self.output_mode == OutputMode::Char {
                    (
                        ((self.counter % 256) as u8 as char).to_string(),
                        "".to_string(),
                        true,
                    )
                } else {
                    (self.counter.to_string() + "\n", "".to_string(), true)
                }
            } else {
                ("".to_string(), "".to_string(), true)
            }
        }
    }
    fn homepage(&self) -> &'static str {
        "https://esolangs.org/wiki/Deadfish"
    }
    fn help(&self) -> &'static str {
        HELP.trim()
    }
}
