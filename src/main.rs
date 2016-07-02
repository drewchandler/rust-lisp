#[macro_use]
extern crate nom;
extern crate rustyline;

use rustyline::Editor;
use nom::IResult::Done;

mod sexp;
mod parser;

fn main() {
    let mut rl = Editor::new();
    let _ = rl.load_history("history.txt");

    loop {
        let readline = rl.readline("rl> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                match parser::sexp(line.as_bytes()) {
                    Done(_, s) => println!("{}", s),
                    _ => println!("Error!")
                };
            },
            _ => {
                println!("exiting...");
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
}
