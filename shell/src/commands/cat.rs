use std::fs;
use std::io::{ self};

pub fn execute(args: &[String]) {
    // If no arguments passed, read from stdin until EOF or empty line
    if args.is_empty() {
            loop {
                let mut input = String::new();
                if let Err(e) = io::stdin().read_line(&mut input) {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
                if input.is_empty() {
                    break;
                }

                print!("{}", input);
            }
    }

    for arg in args {
        if arg == "-" {
            // If argument is "-", read from stdin similarly
            loop {
                let mut input = String::new();
                if let Err(e) = io::stdin().read_line(&mut input) {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
                if input.is_empty() {
                    break;
                }

                print!("{}", input);
            }
            // Read from stdin
        } else {
            // Read from file
            match fs::read_to_string(arg) {
                Ok(contents) => print!("{}", contents),
                Err(e) => eprintln!("cat: {}: {}", arg, e),
            }
        }
    }
}
