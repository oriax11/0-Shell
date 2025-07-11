use std::fs;
use std::path::Path;
pub fn execute(args: &[&str]) {
    // if args.len() != 2 {
    //     eprintln!("mv: missing operand");
    // } else {
    //     let src = args[0];
    //     let dest = args[1];
    //     if let Err(e) = fs::rename(src, dest) {
    //         eprintln!("mv: {}", e);
    //     }
    // }

    let cleaned : Vec<&str>= args.iter().map(|w| w.trim()).filter(|w|!w.is_empty()).collect();

    if args.len() < 2 {
        eprintln!("mv: missing operand");
        return;
    }

    let src = cleaned[0];
    let dist = cleaned[1];

    let path_src =  Path::new(src);
    let path_dest = Path::new(dist);

    if !path_src.exists()  {
         eprintln!("mv: cannot stat '{}': No such file or directory", src);
        return;
    }

    let final_des = if path_dest.is_dir() {
        if let Some(filename) = path_src.file_name() {
            path_dest.join(filename)
        } else {
            eprintln!("mv: failed to get filename from '{}'", src);
            return;
        }
    } else {
        path_dest.to_path_buf()
    };


    match fs::rename(path_src, final_des) {
        Ok(_) => {},
        Err(e) => eprintln!("mv: error moving: {}", e)
    }
}
