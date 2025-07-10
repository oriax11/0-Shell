mod parse;
mod pwd;
mod cd;
mod ls;
mod command;
mod cat;
mod echo;

use libc::{read, STDIN_FILENO};
use std::io;
use std::io::Write;

fn main() {
    let mut buffer = [0u8; 1024];

    loop {
        // Print the prompt manually
        print!("$ ");
        io::stdout().flush().unwrap();

        unsafe {
            let bytes_read = read(STDIN_FILENO, buffer.as_mut_ptr() as *mut _, buffer.len());

            if bytes_read < 0 {
                eprintln!("Error: {}", io::Error::last_os_error());
                break;
            }

            if bytes_read == 0 {
                println!("Ctrl-D");
                break;
            }

            let line = std::str::from_utf8(&buffer[..bytes_read as usize])
                .unwrap_or("")
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();

            if line.is_empty() {
                continue;
            }

            // You may implement Ctrl-C handling with signal hooks, but libc::read itself won't catch it
            parse::parse_data(&line);
        }
    }
}
