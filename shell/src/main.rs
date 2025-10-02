use std::io::{ self, Write };
use std::env;
use std::path::PathBuf;
use std::process::Command;

mod commands;

enum ContinuationReason {
    None,
    Backslash,
    Quote,
}

fn parse_line(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes: Option<char> = None;
    let mut is_escaped = false;

    for c in line.chars() {
        if is_escaped {
            current_arg.push(c);
            is_escaped = false;
            continue;
        }

        if c == '\\' {
            is_escaped = true;
            continue;
        }

        match in_quotes {
            Some(quote_char) => {
                if c == quote_char {
                    in_quotes = None;
                } else {
                    current_arg.push(c);
                }
            }
            None => {
                match c {
                    '\'' | '"' => {
                        in_quotes = Some(c);
                    }
                    _ if c.is_whitespace() => {
                        if !current_arg.is_empty() {
                            args.push(current_arg.clone());
                            current_arg.clear();
                        }
                    }
                    _ => {
                        current_arg.push(c);
                    }
                }
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn needs_continuation(input: &str) -> ContinuationReason {
    let mut single_quotes = 0;
    let mut double_quotes = 0;
    for ch in input.chars() {
        match ch {
            '\'' => {
                single_quotes ^= 1;
            }
            '"' => {
                double_quotes ^= 1;
            }
            _ => {}
        }
    }
    let trailing_backslash = input.trim_end().ends_with('\\');

    if single_quotes == 1 || double_quotes == 1 {
        ContinuationReason::Quote
    } else if trailing_backslash {
        ContinuationReason::Backslash
    } else {
        ContinuationReason::None
    }
}

fn main() {
    loop {
        let current_dir = match env::current_dir() {
            Ok(path) => path,
            Err(_) => {
                PathBuf::from("/")
            }
        };
        let home_dir = home::home_dir().unwrap_or_default(); // home_dir() can return None, but unwrap_or_default() is fine here
        let display_path = if current_dir.starts_with(&home_dir) {
            format!("~/{}", current_dir.strip_prefix(&home_dir).unwrap_or(&current_dir).display())
        } else {
            format!("{}", current_dir.display())
        };

        if let Err(e) = write!(io::stdout(), "{} $ ", display_path) {
            eprintln!("Error writing to stdout: {}", e);
            continue;
        }
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            continue;
        }

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("Error reading from stdin: {}", e);
            continue;
        }
        if input.is_empty() {
            // Ctrl+D was pressed
            break;
        }

        loop {
            match needs_continuation(&input) {
                ContinuationReason::None => {
                    break;
                }
                ContinuationReason::Backslash => {
                    // Remove the trailing backslash and newline
                    input.pop(); // Remove \n
                    if input.ends_with('\r') {
                        input.pop(); // Remove \r if present
                    }
                    input.pop(); // Remove \\
                }
                ContinuationReason::Quote => {
                    // Do not remove the newline for quote continuation
                }
            }

            print!("> ");
            io::stdout().flush().unwrap();
            let mut next_line = String::new();
            let bytes_read = io::stdin().read_line(&mut next_line);

            match bytes_read {
                Ok(0) => {
                    println!("\nSyntax error: Unterminated quoted string");
                    break;
                }
                Ok(_) => {
                    input.push_str(&next_line);
                }
                Err(_) => {
                    println!("\nError reading line.");
                    break;
                }
            }
        }

        let parsed_args = parse_line(&input);
        if parsed_args.is_empty() {
            continue;
        }

        let command = &parsed_args[0];
        let args = &parsed_args[1..];

        match command.as_str() {
            "exit" => {
                break;
            }
            "echo" => commands::echo::execute(args),
            "cd" => commands::cd::execute(args),
            "ls" => commands::ls::execute(args),
            "pwd" => commands::pwd::execute(),
            "cat" => commands::cat::execute(args),
            "cp" => commands::cp::execute(args),
            "mv" => commands::mv::execute(args),
            "mkdir" => commands::mkdir::execute(args),
            "rm" => commands::rm::execute(args),
            _ => {
                if !command.is_empty() {
                    let mut cmd = Command::new(command);
                    cmd.args(args);
                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                eprintln!("Command '{}' exited with status: {}", command, status);
                            }
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::NotFound {
                                eprintln!("Command '{}' not found", command);
                            } else {
                                eprintln!("Error executing command '{}': {}", command, e);
                            }
                        }
                    }
                }
            }
        }
    }
}
