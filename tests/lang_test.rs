use indoc::indoc;
use std::io::Write;
use try_in_browser::lang::{interpret, LangWriter};

struct VecWriter {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl VecWriter {
    fn new() -> Self {
        Self {
            stdout: vec![],
            stderr: vec![],
        }
    }

    fn raw(&self) -> (&[u8], &[u8]) {
        (&self.stdout, &self.stderr)
    }
}

impl LangWriter for VecWriter {
    fn write_both(&mut self, out: &str, err: &str) {
        write!(self.stdout, "{}", out).unwrap();
        write!(self.stderr, "{}", err).unwrap();
    }
}

#[test]
fn test_s10k() {
    VecWriter::init_impls();
    let lang = "S10K";
    let mut writer = VecWriter::new();
    let pgm = "ooo";
    interpret(lang, pgm, "", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out.len(), 10000);
    assert_eq!(err, b"");
}

#[test]
fn test_deadfish() {
    VecWriter::init_impls();
    let lang = "Deadfish";
    let mut writer = VecWriter::new();
    let pgm = "ooo";
    interpret(lang, pgm, "", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"0\n0\n0\n");
    assert_eq!(err, b"");

    let mut writer = VecWriter::new();
    let pgm = indoc!(
        r#"
        iisiiiisiiiiiiiioiiiiiiiiiiiiiiiiiiiiiiiiiiiiioiiiiiiiooiiio
        dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddo
        dddddddddddddddddddddsddoddddddddoiiioddddddoddddddddo
        "#
    );
    interpret(lang, pgm, "", "-o", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"Hello world");
    assert_eq!(err, b"");
}

#[test]
fn test_brainfuck() {
    VecWriter::init_impls();
    let lang = "brainfuck";
    let mut writer = VecWriter::new();
    let pgm = ">>>>--<-<<+[+[<+>--->->->-<<<]>]<<--.<++++++.<<-..<<.<+.>>.>>.<<<.+++.>>.>>-.<<<+.";
    interpret(lang, pgm, "", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"Hello, World!");
    assert_eq!(err, b"");

    let mut writer = VecWriter::new();
    let pgm = ">>>>>+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.";
    interpret(lang, pgm, "", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"Hello, World!");
    assert_eq!(err, b"");

    let mut writer = VecWriter::new();
    let pgm = ",[..,]";
    interpret(lang, pgm, "", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"");
    assert_eq!(err, b"");

    let mut writer = VecWriter::new();
    let pgm = ",[..,]";
    interpret(lang, pgm, "Hello!", "", &mut writer);
    let (out, err) = writer.raw();
    assert_eq!(out, b"HHeelllloo!!");
    assert_eq!(err, b"");
}
