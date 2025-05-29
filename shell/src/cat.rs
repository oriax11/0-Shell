use std::fs;
use std::fs::File;
use std::io::Read;

pub fn handle_cat(rest: &[String]) {
    if rest.is_empty() {
        eprintln!("cat: missing operand");
        return;
    }

    for filename in rest {
        if let Ok(metadata) = fs::metadata(filename) {
            if metadata.is_dir() {
                eprintln!("cat: {}: Is a directory", filename);
                continue;
            }
        }

        match File::open(filename) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Err(err) = file.read_to_string(&mut contents) {
                    eprintln!("cat: {}: {}", filename, err);
                } else {
                    print!("{}", contents);
                }
            }
            Err(err) => {
                eprintln!("cat: {}: {}", filename, err);
            }
        }
    }
}
