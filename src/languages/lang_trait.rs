pub trait Language {
    fn init(&mut self, code: &str, input: &str, args: &str);
    fn step(&mut self) -> (String, String, bool);
    fn homepage(&self) -> &'static str;
    fn help(&self) -> &'static str;
}
