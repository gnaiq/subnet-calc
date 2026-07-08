use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaskError {
    Empty,
    InvalidPrefixLength(u32),
    NotNumber(String),
    InvalidMask(u32),
    WrongSegmentCount(usize),
    SegmentOutOfRange { segment: String, value: u32 },
}

impl fmt::Display for MaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaskError::Empty => write!(f, "输入为空"),
            MaskError::InvalidPrefixLength(v) => {
                write!(f, "非法 CIDR 前缀长度: {}（应在 0-32 之间）", v)
            }
            MaskError::NotNumber(s) => write!(f, "输入不是有效数字: {}", s),
            MaskError::InvalidMask(v) => write!(f, "非法掩码值: 0x{:08X}", v),
            MaskError::WrongSegmentCount(n) => {
                write!(f, "点分十进制段数错误: {}（应为 4 段）", n)
            }
            MaskError::SegmentOutOfRange { segment, value } => {
                write!(
                    f,
                    "段 {} 的值 {} 超出范围（应在 0-255 之间）",
                    segment, value
                )
            }
        }
    }
}

pub fn is_valid_mask(m: u32) -> bool {
    m == 0 || (m | m.wrapping_sub(1)) == 0xFFFFFFFF
}

pub fn mask_to_cidr(m: u32) -> Option<u8> {
    if !is_valid_mask(m) {
        return None;
    }
    Some(m.count_ones() as u8)
}

pub fn cidr_to_mask(cidr: u8) -> u32 {
    if cidr == 0 {
        0
    } else {
        (!0u32) << (32 - cidr)
    }
}

pub fn wildcard_mask(m: u32) -> u32 {
    !m & 0xFFFFFFFF
}

pub fn to_dotted(m: u32) -> String {
    let a = (m >> 24) & 0xFF;
    let b = (m >> 16) & 0xFF;
    let c = (m >> 8) & 0xFF;
    let d = m & 0xFF;
    format!("{}.{}.{}.{}", a, b, c, d)
}

pub fn to_dotted_wildcard(m: u32) -> String {
    to_dotted(wildcard_mask(m))
}

pub fn parse_mask(s: &str) -> Result<u8, MaskError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(MaskError::Empty);
    }

    if s.contains('.') {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return Err(MaskError::WrongSegmentCount(parts.len()));
        }
        let mut bytes: [u32; 4] = [0; 4];
        for (i, part) in parts.iter().enumerate() {
            let v: u32 = part
                .parse()
                .map_err(|_| MaskError::NotNumber(part.to_string()))?;
            if v > 255 {
                return Err(MaskError::SegmentOutOfRange {
                    segment: part.to_string(),
                    value: v,
                });
            }
            bytes[i] = v;
        }
        let m: u32 = (bytes[0] << 24) | (bytes[1] << 16) | (bytes[2] << 8) | bytes[3];
        if !is_valid_mask(m) {
            return Err(MaskError::InvalidMask(m));
        }
        mask_to_cidr(m).ok_or(MaskError::InvalidMask(m))
    } else {
        let v: u32 = s.parse().map_err(|_| MaskError::NotNumber(s.to_string()))?;
        if v > 32 {
            return Err(MaskError::InvalidPrefixLength(v));
        }
        Ok(v as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_mask() {
        assert_eq!(is_valid_mask(0xFFFFFF00), true);
        assert_eq!(is_valid_mask(0), true);
        assert_eq!(is_valid_mask(0xFFFFFFFF), true);
        assert_eq!(is_valid_mask(0xFFFFFF01), false);
        assert_eq!(is_valid_mask(0xFF00FF00), false);
        assert_eq!(is_valid_mask(0x00FFFFFF), false);
    }

    #[test]
    fn test_mask_to_cidr() {
        assert_eq!(mask_to_cidr(0xFFFFFF00), Some(24));
        assert_eq!(mask_to_cidr(0), Some(0));
        assert_eq!(mask_to_cidr(0xFFFFFFFF), Some(32));
        assert_eq!(mask_to_cidr(0xFFFFFF01), None);
        assert_eq!(mask_to_cidr(0xFF00FF00), None);
        assert_eq!(mask_to_cidr(0x80000000), Some(1));
        assert_eq!(mask_to_cidr(0xFE000000), Some(7));
    }

    #[test]
    fn test_cidr_to_mask() {
        assert_eq!(cidr_to_mask(0), 0);
        assert_eq!(cidr_to_mask(8), 0xFF000000);
        assert_eq!(cidr_to_mask(16), 0xFFFF0000);
        assert_eq!(cidr_to_mask(24), 0xFFFFFF00);
        assert_eq!(cidr_to_mask(32), 0xFFFFFFFF);
        assert_eq!(cidr_to_mask(1), 0x80000000);
        assert_eq!(cidr_to_mask(31), 0xFFFFFFFE);
    }

    #[test]
    fn test_wildcard_mask() {
        assert_eq!(wildcard_mask(0xFFFFFF00), 0x000000FF);
        assert_eq!(wildcard_mask(0), 0xFFFFFFFF);
        assert_eq!(wildcard_mask(0xFFFFFFFF), 0);
    }

    #[test]
    fn test_to_dotted() {
        assert_eq!(to_dotted(0xFFFFFF00), "255.255.255.0");
        assert_eq!(to_dotted(0), "0.0.0.0");
        assert_eq!(to_dotted(0xFFFFFFFF), "255.255.255.255");
    }

    #[test]
    fn test_to_dotted_wildcard() {
        assert_eq!(to_dotted_wildcard(0xFFFFFF00), "0.0.0.255");
        assert_eq!(to_dotted_wildcard(0), "255.255.255.255");
    }

    #[test]
    fn test_parse_mask_valid() {
        assert_eq!(parse_mask("24"), Ok(24));
        assert_eq!(parse_mask("0"), Ok(0));
        assert_eq!(parse_mask("32"), Ok(32));
        assert_eq!(parse_mask("255.255.255.0"), Ok(24));
        assert_eq!(parse_mask("0.0.0.0"), Ok(0));
        assert_eq!(parse_mask("255.255.255.255"), Ok(32));
        assert_eq!(parse_mask("255.255.255.252"), Ok(30));
    }

    #[test]
    fn test_parse_mask_invalid() {
        assert_eq!(parse_mask("33"), Err(MaskError::InvalidPrefixLength(33)));
        assert_eq!(
            parse_mask("255.255.255.1"),
            Err(MaskError::InvalidMask(0xFFFFFF01))
        );
        assert_eq!(
            parse_mask("abc"),
            Err(MaskError::NotNumber("abc".to_string()))
        );
        assert_eq!(parse_mask(""), Err(MaskError::Empty));
        assert_eq!(parse_mask("1.2.3"), Err(MaskError::WrongSegmentCount(3)));
        assert_eq!(
            parse_mask("256.0.0.0"),
            Err(MaskError::SegmentOutOfRange {
                segment: "256".to_string(),
                value: 256
            })
        );
    }
}
