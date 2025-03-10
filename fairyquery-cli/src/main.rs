mod history;
use history::InputHistory;
use std::io::{Write, stderr, stdin, stdout};

fn main() {
    let mut input_history = InputHistory::new();
    loop {
        print!("gingerbread> ");
        stdout().flush().unwrap();
        // read a line from stdin
        stdin()
            .read_line(input_history.push())
            .expect("Failed to read line");
        // remove the newline character
        let input = input_history.get();
        // if the input is empty, skip the rest of the loop
        if input.is_empty() {
            continue;
        }
        // if the input is "exit", break the loop
        if input == "\\q" {
            break;
        } else {
            eprintln!("Unknown command: {}", input);
            stderr().flush().unwrap();
        }
    }
}
