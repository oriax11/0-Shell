use std::fs;

pub fn execute(args: &[&str]) {
    if args.len() != 2 {
        eprintln!("mv: missing operand");
    } else {
        let src = args[0];
        let dest = args[1];
        if let Err(e) = fs::rename(src, dest) {
            eprintln!("mv: {}", e);
        }
    }
}
