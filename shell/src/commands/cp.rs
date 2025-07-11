use std::fs;
use std::path::Path;

pub fn execute(args: &[&str]) {
    // println!("{}", args.len());
    // if args.len() != 2 {
    //     eprintln!("cp: missing operand");
    // } else {
    //     let src = args[0];
    //     let dest = args[1];
    //     println!("{} {}", src, dest);
    //     if let Err(e) = fs::copy(src, dest) {
    //         eprintln!("cp: {}", e);
    //     }
    // }

    if args.len() < 2 {
        eprintln!("cp: missing operand");
        eprintln!("usage: cp <source> <destination>");
        return;
    }

    let src = args[0];
    let dest = args[1];

    let src_path = Path::new(src);
    let dest_path = Path::new(dest);

    if !src_path.exists() {
        eprintln!("cp: cannot stat '{}': No such file or directory", src);
        return;
    }

    // If destination is a directory, append the source filename to it
    let final_dest = if dest_path.is_dir() {
        if let Some(filename) = src_path.file_name() {
            dest_path.join(filename)
        } else {
            eprintln!("cp: failed to get filename from '{}'", src);
            return;
        }
    } else {
        dest_path.to_path_buf()
    };

    match fs::copy(src_path, final_dest) {
        Ok(_) => {},
        Err(e) => eprintln!("cp: error copying: {}", e),
    }
}
