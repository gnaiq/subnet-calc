use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpError {
    Empty,
    WrongSegmentCount(usize),
    SegmentNotNumber(String),
    SegmentOutOfRange { segment: String, value: u32 },
    LeadingZero(String),
}

impl fmt::Display for IpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpError::Empty => write!(f, "输入为空"),
            IpError::WrongSegmentCount(n) => {
                write!(f, "IP 地址段数不为 4，实际段数为 {}", n)
            }
            IpError::SegmentNotNumber(s) => write!(f, "段 \"{}\" 不是合法数字", s),
            IpError::SegmentOutOfRange { segment, value } => {
                write!(f, "段 \"{}\" 的值 {} 超出范围（0-255）", segment, value)
            }
            IpError::LeadingZero(s) => write!(f, "段 \"{}\" 存在前导零", s),
        }
    }
}

impl std::error::Error for IpError {}

pub fn parse_ipv4(s: &str) -> Result<u32, IpError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(IpError::Empty);
    }

    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return Err(IpError::WrongSegmentCount(parts.len()));
    }

    let mut result: u32 = 0;
    for part in parts {
        if part.is_empty() {
            return Err(IpError::SegmentNotNumber(part.to_string()));
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(IpError::LeadingZero(part.to_string()));
        }
        let value: u32 = part
            .parse()
            .map_err(|_| IpError::SegmentNotNumber(part.to_string()))?;
        if value > 255 {
            return Err(IpError::SegmentOutOfRange {
                segment: part.to_string(),
                value,
            });
        }
        result = (result << 8) | value;
    }

    Ok(result)
}

pub fn is_valid_ipv4(s: &str) -> bool {
    parse_ipv4(s).is_ok()
}

pub fn to_dotted(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF
    )
}

pub fn to_binary(ip: u32) -> String {
    format!(
        "{:08b}.{:08b}.{:08b}.{:08b}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF
    )
}

pub fn to_hex(ip: u32) -> String {
    format!("{:08X}", ip)
}

pub fn to_integer(ip: u32) -> String {
    ip.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpClass {
    ClassA,
    ClassB,
    ClassC,
    ClassD,
    ClassE,
}

impl IpClass {
    pub fn description(&self) -> &'static str {
        match self {
            IpClass::ClassA => "A类地址",
            IpClass::ClassB => "B类地址",
            IpClass::ClassC => "C类地址",
            IpClass::ClassD => "组播地址",
            IpClass::ClassE => "保留地址",
        }
    }
}

pub fn ip_class(ip: u32) -> IpClass {
    let first = ((ip >> 24) & 0xFF) as u8;
    match first {
        0..=127 => IpClass::ClassA,
        128..=191 => IpClass::ClassB,
        192..=223 => IpClass::ClassC,
        224..=239 => IpClass::ClassD,
        240..=255 => IpClass::ClassE,
    }
}

pub fn is_private(ip: u32) -> bool {
    let first = (ip >> 24) & 0xFF;
    let second = (ip >> 16) & 0xFF;
    if first == 10 {
        return true;
    }
    if first == 172 && (16..=31).contains(&second) {
        return true;
    }
    if first == 192 && second == 168 {
        return true;
    }
    false
}

pub fn is_loopback(ip: u32) -> bool {
    let first = (ip >> 24) & 0xFF;
    first == 127
}

pub fn is_broadcast(ip: u32) -> bool {
    ip == 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_valid() {
        assert_eq!(parse_ipv4("192.168.1.1"), Ok(0xC0A80101));
        assert_eq!(parse_ipv4("0.0.0.0"), Ok(0));
        assert_eq!(parse_ipv4("255.255.255.255"), Ok(0xFFFFFFFF));
    }

    #[test]
    fn parse_ipv4_invalid() {
        assert_eq!(
            parse_ipv4("999.1.1.1"),
            Err(IpError::SegmentOutOfRange {
                segment: "999".to_string(),
                value: 999
            })
        );
        assert_eq!(parse_ipv4("1.2.3"), Err(IpError::WrongSegmentCount(3)));
        assert_eq!(
            parse_ipv4("1.2.3.4.5"),
            Err(IpError::WrongSegmentCount(5))
        );
        assert_eq!(
            parse_ipv4("a.b.c.d"),
            Err(IpError::SegmentNotNumber("a".to_string()))
        );
        assert_eq!(
            parse_ipv4("01.2.3.4"),
            Err(IpError::LeadingZero("01".to_string()))
        );
        assert_eq!(parse_ipv4(""), Err(IpError::Empty));
    }

    #[test]
    fn is_valid_ipv4_works() {
        assert!(is_valid_ipv4("192.168.1.1"));
        assert!(!is_valid_ipv4("999.1.1.1"));
        assert!(!is_valid_ipv4(""));
    }

    #[test]
    fn to_binary_works() {
        assert_eq!(
            to_binary(0xC0A80101),
            "11000000.10101000.00000001.00000001"
        );
    }

    #[test]
    fn to_hex_works() {
        assert_eq!(to_hex(0xC0A80101), "C0A80101");
    }

    #[test]
    fn to_integer_works() {
        assert_eq!(to_integer(0xC0A80101), "3232235777");
    }

    #[test]
    fn to_dotted_works() {
        assert_eq!(to_dotted(0xC0A80101), "192.168.1.1");
        assert_eq!(to_dotted(0), "0.0.0.0");
        assert_eq!(to_dotted(0xFFFFFFFF), "255.255.255.255");
    }

    #[test]
    fn ip_class_works() {
        assert_eq!(ip_class(0x0A000001), IpClass::ClassA);
        assert_eq!(ip_class(0xAC100001), IpClass::ClassB);
        assert_eq!(ip_class(0xC0A80101), IpClass::ClassC);
        assert_eq!(ip_class(0xE0000001), IpClass::ClassD);
        assert_eq!(ip_class(0xF0000001), IpClass::ClassE);

        assert_eq!(ip_class(0x7F000001), IpClass::ClassA);
        assert_eq!(ip_class(0x80000001), IpClass::ClassB);
        assert_eq!(ip_class(0xBF000001), IpClass::ClassB);
        assert_eq!(ip_class(0xC0000001), IpClass::ClassC);
        assert_eq!(ip_class(0xDF000001), IpClass::ClassC);
        assert_eq!(ip_class(0xE0000001), IpClass::ClassD);
        assert_eq!(ip_class(0xEF000001), IpClass::ClassD);
        assert_eq!(ip_class(0xF0000001), IpClass::ClassE);
        assert_eq!(ip_class(0xFF000001), IpClass::ClassE);
    }

    #[test]
    fn ip_class_description_works() {
        assert_eq!(IpClass::ClassA.description(), "A类地址");
        assert_eq!(IpClass::ClassB.description(), "B类地址");
        assert_eq!(IpClass::ClassC.description(), "C类地址");
        assert_eq!(IpClass::ClassD.description(), "组播地址");
        assert_eq!(IpClass::ClassE.description(), "保留地址");
    }

    #[test]
    fn is_private_works() {
        assert!(is_private(0x0A000001));
        assert!(is_private(0xAC100001));
        assert!(is_private(0xAC1FFF01));
        assert!(!is_private(0xAC200001));
        assert!(is_private(0xC0A80101));
        assert!(!is_private(0x08080808));
    }

    #[test]
    fn is_loopback_works() {
        assert!(is_loopback(0x7F000001));
        assert!(!is_loopback(0x7E000001));
    }

    #[test]
    fn is_broadcast_works() {
        assert!(is_broadcast(0xFFFFFFFF));
        assert!(!is_broadcast(0xFFFFFFFE));
    }
}
