use std::fs;

pub fn execute(args: &[&str]) {
    for arg in args {
        if let Err(e) = fs::create_dir(arg) {
            eprintln!("mkdir: {}: {}", arg, e);
        }
    }
}
