use std::env;
use std::path::PathBuf;

pub fn execute(args: &[String]) {
    // Clean up input arguments: trim whitespace and remove empty strings
    let cleaned: Vec<&str> = args.iter().map(|s| s.as_str().trim()).filter(|s| !s.is_empty()).collect();
    let target = cleaned.get(0).unwrap_or(&"~");

    // Resolve path with proper tilde expansion
    let path = if *target == "~" || target.starts_with("~/") {
        match home::home_dir() {
            Some(mut home) => {
                if *target == "~" {
                    home
                } else {
                    // Handle ~/path/to/somewhere
                    home.push(&target[2..]); // Skip "~/"
                    home
                }
            }
            None => {
                eprintln!("cd: HOME not set");
                return;
            }
        }
    } else {
        PathBuf::from(target)
    };

    // Attempt to change current directory, print error on failure
    if let Err(e) = env::set_current_dir(&path) {
        eprintln!("cd: {}: {}", target, e);
    }
}
