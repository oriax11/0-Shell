mod parse;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::parse::*;

fn main() {
    let mut ligne = Editor::<(), _>::new().unwrap();
    loop {
        let readline = ligne.readline("$ ");
        match readline {
            Ok(line) => {
                let _ = ligne.add_history_entry(line.as_str()); // pour ↑ ↓
                parse_data(&line)
            },
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            },
            Err(err) => {
                println!("Erreur : {:?}", err);
                break;
            }
        }
    }
}
