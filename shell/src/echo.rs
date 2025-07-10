use libc::write;
use libc::STDOUT_FILENO;

pub fn handle_echo(rest: &[String]) {
    // Join arguments with spaces and append newline
    let mut output = rest.join(" ");
    output.push('\n');

    // Write to stdout using libc
    unsafe {
        write(
            STDOUT_FILENO,
            output.as_ptr() as *const _,
            output.len(),
        );
    }
}
