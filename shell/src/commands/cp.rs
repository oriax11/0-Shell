use std::fs;

pub fn execute(args: &[&str]) {
    if args.len() != 2 {
        eprintln!("cp: missing operand");
    } else {
        let src = args[0];
        let dest = args[1];
        if let Err(e) = fs::copy(src, dest) {
            eprintln!("cp: {}", e);
        }
    }
}
