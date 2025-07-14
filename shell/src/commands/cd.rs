use std::env;
use std::path::PathBuf;

pub fn execute(args: &[String]) {
    // Clean up input arguments: trim whitespace and remove empty strings
    let cleaned: Vec<&str> = args.iter().map(|s| s.as_str().trim()).filter(|s| !s.is_empty()).collect();
    let target = cleaned.get(0).unwrap_or(&"~");
    // Resolve path: if "~", use home directory; else use given path
    let path = if *target == "~" {
        home::home_dir().unwrap_or_default()
    } else {
        PathBuf::from(target)
    };
    // Attempt to change current directory, print error on failure
    if let Err(_) = env::set_current_dir(&path) {
        eprintln!("cd: no matches found: {}",target );
    }
}
