use std::collections::HashMap;

use indoc::indoc;

use super::LangWriter;

pub const NAME: &str = "brainfuck";
pub const HOMEPAGE: &str = "https://esolangs.org/wiki/Brainfuck";
pub const HELP: &str = indoc!(
    r#"
    brainfuck (https://esolangs.org/wiki/Brainfuck)
    Accepted arguments:
    -h    Show this help and exit

    +    Increment cell
    -    Increment cell
    >    Move pointer right
    <    Move pointer left
    .    Output character value of cell
    ,    Read a character as an integer
    [    Start of while loop
    ]    End of loop
    "#
);

pub fn interpret<T: LangWriter>(pgm_str: &str, input_str: &str, _args: &str, writer: &mut T) {
    let pgm = String::from(pgm_str).into_bytes();
    let mut input = input_str.bytes();
    let mut ind = 0usize;

    let mut pos = 0usize;
    let zeroes = vec![0u8; 100];
    let mut tape = zeroes.to_vec();

    //Keys are the indices of loop starts, values are indices of loop ends
    let mut loops = HashMap::<usize, usize>::new();
    //The indices of the starts of the loops that it's currently in
    //This `Vec` is reused for the actual interpreting too
    let mut loop_starts = Vec::<usize>::new();

    //Load the indices of the `[`'s and `]`'s into `loops`
    while ind < pgm.len() {
        match pgm[ind] {
            b'[' => loop_starts.push(ind),
            b']' => {
                match loop_starts.pop() {
                    Some(loop_start) => {
                        loops[loop_start] = ind
                    },
                    None => {
                        writer.terminate_with_error(
                            &*format!(
                                "Extra `]` found at index {}",
                                ind
                            )
                        );
                        return;
                    }
                }
            }
            _ => {}
        }
        ind += 1
    }

    //Handle unclosed loops
    if !loop_starts.is_empty() {
        writer.terminate_with_error(
            &*format!(
                "Error: Missing closing `]`'s to correspond with `[`'s at indices {:?}",
                loops
            )
        );
        return;
    }

    while ind < pgm.len() {
        let curr_cmd = pgm[ind];
        ind += 1;
        match curr_cmd {
            b'+' => {
                if tape[pos] == 256 {
                    tape[pos] = 0
                } else {
                    tape[pos] += 1
                }
            }
            b'-' => {
                if tape[pos] == 0 {
                    tape[pos] = 256
                } else {
                    tape[pos] -= 1
                }
            }
            b'>' => {
                pos += 1;
                if pos == tape.len() {
                    tape.extend(zeroes.iter())
                }
            }
            b'<' => {
                pos -= 1;
                if pos < 0 {
                    writer.terminate_with_error(
                        &*format!(
                            "Error on `<` at index {}: Reached left end of tape",
                            ind
                        )
                    );
                    return;
                }
            }
            b'.' => {
                let out = tape[pos] as char;
                writer.write_out(&*out.to_string())
            }
            b',' => {
                match input.next() {
                    Some(char) => {
                        tape[pos] = char
                    }
                    None => {
                        writer.terminate_with_error(
                            &*format!(
                                "Error on `,` at index {}: No input left",
                                ind
                            )
                        );
                        return;
                    }
                }
            }
            b'[' => {
                if tape[pos] == 0 {
                    //Jump to one command after the end of the loop
                    ind = loops[ind] + 1
                } else {
                    loop_starts.push(ind)
                }
            }
            b']' => {
                //Jump to the start of this loop, which is the last loop
                //We can unwrap without fear because the loops have been
                //checked in the previous while loop
                ind = loop_starts.pop().unwrap()
            }
            _ => {} //This is a comment, don't do anything
        }
    }
}