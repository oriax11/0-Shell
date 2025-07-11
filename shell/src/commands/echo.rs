pub fn execute(args: &[&str]) {
    let raw = args.join(" ");
    let stripped = raw.trim_matches(|c| c == '\'' || c == '"');

    let interpreted = interpret_escapes(stripped);

    println!("{}", interpreted);
}

// Helper function to interpret escape sequences
fn interpret_escapes(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('n') => {
                    output.push('\n');
                    chars.next();
                }
                Some('t') => {
                    output.push('\t');
                    chars.next();
                }
                Some('\\') => {
                    output.push('\\');
                    chars.next();
                }
                Some('"') => {
                    output.push('"');
                    chars.next();
                }
                Some('\'') => {
                    output.push('\'');
                    chars.next();
                }
                Some(other) => {
                    output.push('\\');
                    output.push(*other);
                    chars.next();
                }
                None => output.push('\\'),
            }
        } else {
            output.push(ch);
        }
    }

    output
}
