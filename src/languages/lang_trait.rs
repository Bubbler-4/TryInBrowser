pub trait Language {
    fn init(&mut self, code: &str, input: &str, args: &str);
    fn step(&mut self) -> (String, String, bool);
}
