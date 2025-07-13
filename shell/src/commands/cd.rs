use std::env;
use std::path::PathBuf;

pub fn execute(args: &[String]) {
    let cleaned: Vec<&str> = args.iter().map(|s| s.as_str().trim()).filter(|s| !s.is_empty()).collect();
    let target = cleaned.get(0).unwrap_or(&"~");
    let path = if *target == "~" {
        home::home_dir().unwrap_or_default()
    } else {
        PathBuf::from(target)
    };
    if let Err(_) = env::set_current_dir(&path) {
        eprintln!("cd: no matches found: {}",target );
    }
}
