use std::env;
use std::io::{self, Write};

pub fn execute(args: &[&str]) {
    let mut raw = args.join(" ");
    if needs_continuation(&raw) {
        raw = collect_multiline(raw);
    }

    let expanded = handle_tilde(&raw);

    let final_output = if is_fully_quoted(&expanded.trim()) {
        let stripped = strip_surrounding_quotes(&expanded.trim());
        interpret_escapes(&stripped)
        
    } else {
        let cleaned = collapse_whitespace(&expanded);
        interpret_escapes(&cleaned)
    };

    println!("{}", final_output);
}

// Detect if the input needs multiline continuation
fn needs_continuation(input: &str) -> bool {
    let mut single_quotes = 0;
    let mut double_quotes = 0;
    for ch in input.chars() {
        match ch {
            '\'' => single_quotes ^= 1,
            '"' => double_quotes ^= 1,
            _ => {}
        }
    }

    let trailing_backslash = input.trim_end().ends_with('\\');
    single_quotes == 1 || double_quotes == 1 || trailing_backslash
}

fn collect_multiline(mut current: String) -> String {
    current.push('\n');
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut next_line = String::new();
        if io::stdin().read_line(&mut next_line).is_err() {
            break;
        }
        current.push_str(&next_line);

        if !needs_continuation(&current) {
            break;
        }
    }
    // print!("{}", current);

    current
}

// Collapse spaces only when unquoted
fn collapse_whitespace(input: &str) -> String {
    input
        .split(" ")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

// Check if fully quoted (preserve spaces, no trim)
fn is_fully_quoted(input: &str) -> bool {
    (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''))
}

// Strip matching surrounding quotes
fn strip_surrounding_quotes(input: &str) -> String {
    if input.len() >= 2 {
        let first = input.chars().next().unwrap();
        let last = input.chars().last().unwrap();
        if (first == '\'' && last == '\'') || (first == '"' && last == '"') {
            return input[1..input.len() - 1].to_string();
        }
    }
    input.to_string()
}

// Tilde expansion only if unquoted
fn handle_tilde(input: &str) -> String {
    if input.trim() == "~" {
        env::var("HOME").unwrap_or_else(|_| "~".to_string())
    } else {
        input.to_string()
    }
}

// Handle escape sequences: \n, \t, \\, \"
fn interpret_escapes(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('n') => {
                    result.push('\n');
                    chars.next();
                }
                Some('t') => {
                    result.push('\t');
                    chars.next();
                }
                Some('\\') => {
                    result.push('\\');
                    chars.next();
                }
                Some('"') => {
                    result.push('"');
                    chars.next();
                }
                Some('\'') => {
                    result.push('\'');
                    chars.next();
                }
                Some(c) => {
                    result.push('\\');
                    result.push(*c);
                    chars.next();
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}
