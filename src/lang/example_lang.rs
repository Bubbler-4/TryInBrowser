use super::LangWriter;
use indoc::indoc;

pub const NAME: &str = "ExampleLang";
pub const HOMEPAGE: &str = "https://example.com";
pub const HELP: &str = indoc!(
    r#"
    An example language for debugging purposes.

    program == "lang": Print some chars on stdout and stderr, and halt (fast).
    program == "slow": Print some chars on stdout and stderr, and halt (slow).
    program == "crasher": Print some chars but crash after a while.
    program == "looper": Print things very slowly, forever.
    program == "talker": Print things very fast, exceeding the output limit.
    "#
);

pub fn interpret<T: LangWriter>(pgm: &str, input: &str, args: &str, writer: &mut T) {
    match pgm {
        "lang" => interpret_lang(pgm, input, args, writer),
        "slow" => interpret_slow(pgm, input, args, writer),
        "crasher" => interpret_crasher(pgm, input, args, writer),
        "looper" => interpret_looper(pgm, input, args, writer),
        "talker" => interpret_talker(pgm, input, args, writer),
        _ => writer.write_err(&format!("Unrecognized program: {}", pgm)),
    }
}

fn interpret_lang<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    for i in 0..40 {
        writer.write_both("S", &format!("{}", i));
    }
    writer.terminate();
}

fn interpret_slow<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    for i in 0..400_000_000 {
        if i % 10_000_000 == 0 {
            writer.write_both("S", &format!("{}", i / 10_000_000 % 10));
        }
    }
    writer.terminate();
}

fn interpret_crasher<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    for i in 0..400_000_000 {
        if i % 10_000_000 == 0 {
            writer.write_both("S", &format!("{}", i / 10_000_000 % 10));
        }
    }
    panic!("wtf");
}

fn interpret_looper<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    let mut i = 0;
    loop {
        if i % 10_000_000 == 0 {
            writer.write_both("S", &format!("{}", i / 10_000_000 % 10));
            if i >= 100_000_000 {
                i = 0;
            }
        }
        i += 1;
    }
}

fn interpret_talker<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    let mut i = 0;
    loop {
        if i % 100 == 0 {
            writer.write_out("S");
        }
        if i % 100_000 == 0 {
            writer.write_err(&format!("{}", i / 100_000 % 10));
            if i >= 100_000_000 {
                i = 0;
            }
        }
        i += 1;
    }
}
