pub fn normalize_input(s: &str) -> String {
    let converted: String = s
        .chars()
        .map(|c| {
            let code = c as u32;
            match code {
                0xFF10..=0xFF19 | 0xFF21..=0xFF3A | 0xFF41..=0xFF5A => {
                    char::from_u32(code - 0xFEE0).unwrap()
                }
                0xFF0E => '.',
                0xFF0F => '/',
                0x3000 => ' ',
                _ => c,
            }
        })
        .collect();

    let trimmed = converted.trim();

    let mut compressed = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for c in trimmed.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                compressed.push(' ');
            }
            prev_space = true;
        } else {
            compressed.push(c);
            prev_space = false;
        }
    }

    compressed
        .split('/')
        .map(|p| p.trim())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_digits() {
        assert_eq!(normalize_input("１９２．１６８．１．１／２４"), "192.168.1.1/24");
    }

    #[test]
    fn multiple_spaces() {
        assert_eq!(
            normalize_input("192.168.1.1   255.255.255.0"),
            "192.168.1.1 255.255.255.0"
        );
    }

    #[test]
    fn spaces_around_slash() {
        assert_eq!(normalize_input("10.0.0.1 / 8"), "10.0.0.1/8");
    }

    #[test]
    fn mixed_input() {
        assert_eq!(
            normalize_input("  １９２．１６８．１．１   ／   ２４  "),
            "192.168.1.1/24"
        );
    }

    #[test]
    fn trim_whitespace() {
        assert_eq!(normalize_input("  192.168.1.1  "), "192.168.1.1");
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_input(""), "");
    }
}
