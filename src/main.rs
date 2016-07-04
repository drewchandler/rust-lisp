#[macro_use]
extern crate nom;
extern crate rustyline;

use rustyline::Editor;
use nom::IResult::Done;

mod sexp;
mod parser;
mod env;

fn main() {
    let mut rl = Editor::new();
    let _ = rl.load_history("history.txt");
    let root = env::env_new(None);

    loop {
        let readline = rl.readline("rl> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                match parser::sexp(line.as_bytes()) {
                    Done(_, s) => {
                        match s.eval(&root) {
                            Ok(s) => println!("{}", s),
                            Err(e) => println!("ERROR: {}", e)
                        };
                    },
                    _ => println!("ERROR: Parse error")
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
