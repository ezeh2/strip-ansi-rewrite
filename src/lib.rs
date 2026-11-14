//! Remove ANSI escape sequences from UTF-8 text.

/// Return `input` with ANSI control sequences removed.
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    if !bytes.iter().any(|&byte| byte == 0x1B || byte == 0x9B) {
        return input.to_owned();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < bytes.len() {
        if let Some(end) = sequence_end(bytes, index) {
            index = end;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(output).expect("input was valid UTF-8 and only complete sequences were removed")
}

fn sequence_end(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes[start] {
        0x9B => csi_end(bytes, start + 1),
        // C1 controls are encoded as two-byte UTF-8 sequences in a Rust `&str`.
        0xC2 if bytes.get(start + 1) == Some(&0x9B) => csi_end(bytes, start + 2),
        0x1B => escape_end(bytes, start),
        _ => None,
    }
}

fn csi_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() && (0x30..=0x3F).contains(&bytes[index]) {
        index += 1;
    }
    while index < bytes.len() && (0x20..=0x2F).contains(&bytes[index]) {
        index += 1;
    }
    if index < bytes.len() && (0x40..=0x7E).contains(&bytes[index]) {
        Some(index + 1)
    } else {
        None
    }
}

fn escape_end(bytes: &[u8], start: usize) -> Option<usize> {
    let next = *bytes.get(start + 1)?;
    match next {
        b'[' => csi_end(bytes, start + 2),
        b']' => osc_end(bytes, start + 2),
        b'P' | b'X' | b'^' | b'_' | b'k' => control_string_end(bytes, start + 2),
        b' ' ..= b'/' => {
            let mut index = start + 1;
            while index < bytes.len() && (0x20..=0x2F).contains(&bytes[index]) {
                index += 1;
            }
            if index < bytes.len() && (0x30..=0x7E).contains(&bytes[index]) {
                Some(index + 1)
            } else {
                None
            }
        }
        b'7' | b'8' | b'=' | b'>' | b'<' | b'~' | b'|' | b'}' => Some(start + 2),
        _ => None,
    }
}

fn osc_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            0x07 | 0x9C => return Some(index + 1),
            0xC2 if bytes.get(index + 1) == Some(&0x9C) => return Some(index + 2),
            0x18 | 0x1A | 0x1B | 0x80..=0x9F => return None,
            0xC2 if bytes.get(index + 1).is_some_and(|byte| (0x80..=0x9F).contains(byte)) => {
                return None
            }
            _ => index += 1,
        }
    }
    None
}

fn control_string_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index + 1 < bytes.len() {
        if bytes[index] == 0x1B && bytes[index + 1] == b'\\' {
            return Some(index + 2);
        }
        if bytes[index] == 0x9C {
            return Some(index + 1);
        }
        if bytes[index] == 0xC2 && bytes.get(index + 1) == Some(&0x9C) {
            return Some(index + 2);
        }
        if bytes[index] == 0x18 || bytes[index] == 0x1A || bytes[index] == 0x1B {
            return None;
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn strips_basic_csi_styles() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strips_osc_hyperlinks() {
        assert_eq!(strip_ansi("\x1b]8;;https://example.com\x07link\x1b]8;;\x07"), "link");
    }

    #[test]
    fn preserves_plain_unicode() {
        assert_eq!(strip_ansi("hello, 世界 🌍"), "hello, 世界 🌍");
    }
}
