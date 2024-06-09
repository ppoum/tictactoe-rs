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

/// Reads from stdin until we receive a valid index from the specified list of options.
/// Adds `(1-n)` to the end of the prompt, where `n` is the number of options.
/// The number returned is 0-indexed, meaning the true range is `[0, n)`.
pub fn read_list(prompt: impl AsRef<str>, options: &[impl AsRef<str>]) -> usize {
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();

    // Print list
    for (i, item) in options.iter().enumerate() {
        println!("{}) {}", i + 1, item.as_ref());
    }

    loop {
        print!("{} (1-{}): ", prompt.as_ref(), options.len());
        io::stdout().flush().unwrap();
        stdin
            .read_line(&mut buffer)
            .expect("Error reading from stdin");

        let input = match buffer.trim().parse::<usize>() {
            Ok(i) => i,
            Err(_) => {
                println!("Invalid value");
                buffer = String::new();
                continue;
            }
        };

        if (1..=options.len()).contains(&input) {
            return input - 1;
        } else {
            buffer = String::new();
            println!("Choice not within bounds.");
        }
    }
}
