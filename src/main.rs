use std::io::{self, Read};

fn main() -> io::Result<()> {
    let input = std::env::args_os().skip(1).collect::<Vec<_>>();
    let text = if input.is_empty() {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        text
    } else {
        input
            .iter()
            .map(|argument| argument.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    };

    print!("{}", strip_ansi::strip_ansi(&text));
    Ok(())
}
