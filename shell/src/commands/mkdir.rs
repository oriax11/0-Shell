use std::fs;

pub fn execute(args: &[String]) {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return;
    }

    // Check for unsupported flags
    for arg in args {
        let trimmed = arg.trim();
        if trimmed.starts_with('-') {
            eprintln!("mkdir: invalid option -- '{}'", trimmed.trim_start_matches('-'));
            eprintln!("mkdir does not support flags");
            return;
        }
    }

    for arg in args {
        let trimmed = arg.trim();
        if !trimmed.is_empty() {
            if let Err(e) = fs::create_dir(trimmed) {
                eprintln!("mkdir: {}: {}", trimmed, e);
            }
        }
    }
}
