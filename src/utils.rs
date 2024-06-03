use std::io::{self, BufRead, Write};

/// Reads from stdin until we receive a boolean answer. Appends either `[Y/n]` or `[y/N]` to the
/// prompt based on the value of the `default` argument.
pub fn read_bool(prompt: impl AsRef<str>, default: bool) -> bool {
    let prompt_extra = if default { "[Y/n]: " } else { "[y/N]: " };
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();
    loop {
        print!("{} {}", prompt.as_ref(), prompt_extra);
        io::stdout().flush().unwrap();
        stdin
            .read_line(&mut buffer)
            .expect("Error reading from stdin");

        match buffer.trim().to_lowercase().as_ref() {
            "" => return default,
            "yes" | "y" | "1" => return true,
            "no" | "n" | "0" => return false,
            _ => {}
        }

        println!("Invalid value");
        buffer = String::new();
    }
}
