use std::env;
use std::path::PathBuf;

pub fn execute(args: &[&str]) {
    //println!("{:?}", args);
    let cleaned :Vec<&str> = args.iter().map(|w| w.trim()).filter(|w| !w.is_empty()).collect(); 
    //println!("{:?}", cleaned);
    let target = cleaned.get(0).unwrap_or(&"~");
    let path = if *target == "~" {
        home::home_dir().unwrap_or_default()
    } else {
        PathBuf::from(target)
    };
    if let Err(e) = env::set_current_dir(&path) {
        eprintln!("cd: {}", e);
    }
}
