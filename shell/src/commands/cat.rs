use std::fs;

pub fn execute(args: &[&str]) {
    for arg in args {
        match fs::read_to_string(arg) {
            Ok(contents) => print!("{}", contents),
            Err(e) => eprintln!("cat: {}: {}", arg, e),
        }
    }
}
