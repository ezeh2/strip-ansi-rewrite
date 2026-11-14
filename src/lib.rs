//! Remove ANSI escape sequences from UTF-8 text.

/// Return `input` with ANSI control sequences removed.
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    if !bytes.iter().enumerate().any(|(index, &byte)| {
        byte == 0x1B
            || byte == 0x9B
            || (byte == 0xC2
                && bytes
                    .get(index + 1)
                    .is_some_and(|next| matches!(next, 0x90 | 0x98 | 0x9B | 0x9D | 0x9E | 0x9F)))
    }) {
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
        0xC2 if bytes.get(start + 1) == Some(&0x9D) => osc_end(bytes, start + 2),
        0xC2 if matches!(bytes.get(start + 1), Some(0x90 | 0x98 | 0x9E | 0x9F)) => {
            control_string_end(bytes, start + 2)
        }
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
        b']' => {
            if let Some(end) = linux_palette_end(bytes, start) {
                Some(end)
            } else {
                osc_end(bytes, start + 2)
            }
        }
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

fn linux_palette_end(bytes: &[u8], start: usize) -> Option<usize> {
    let kind = *bytes.get(start + 2)?;
    let payload_end = match kind {
        b'R' => start + 3,
        b'P' if start + 10 <= bytes.len()
            && bytes[start + 3..start + 10]
                .iter()
                .all(|byte| byte.is_ascii_hexdigit()) =>
        {
            start + 10
        }
        _ => return None,
    };

    match bytes.get(payload_end) {
        Some(0x07) => Some(payload_end + 1),
        Some(0x1B) if bytes.get(payload_end + 1) == Some(&b'\\') => Some(payload_end + 2),
        _ => Some(payload_end),
    }
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
    fn strips_utf8_encoded_c1_controls() {
        assert_eq!(strip_ansi("\u{009B}31mred\u{009B}0m"), "red");
        assert_eq!(
            strip_ansi("\u{001B}[0m\u{009D}0;title\u{0007}visible"),
            "visible"
        );
        assert_eq!(
            strip_ansi("\u{001B}[0m\u{0090}opaque\u{009C}visible"),
            "visible"
        );
    }

    #[test]
    fn strips_linux_palette_controls_without_a_terminator() {
        assert_eq!(strip_ansi("\x1b]P1234567visible"), "visible");
        assert_eq!(strip_ansi("\x1b]Rvisible"), "visible");
    }

    #[test]
    fn preserves_plain_unicode() {
        assert_eq!(strip_ansi("hello, 世界 🌍"), "hello, 世界 🌍");
    }
}
