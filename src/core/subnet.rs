use std::fmt;

use crate::core::ip::{self, IpClass, IpError};
use crate::core::mask::{self, MaskError};
use crate::core::normalize::normalize_input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubnetError {
    IpError(IpError),
    MaskError(MaskError),
    NoDefaultMask(IpClass),
    Empty,
}

impl fmt::Display for SubnetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubnetError::IpError(e) => write!(f, "{}", e),
            SubnetError::MaskError(e) => write!(f, "{}", e),
            SubnetError::NoDefaultMask(c) => {
                write!(f, "{}无默认子网掩码", c.description())
            }
            SubnetError::Empty => write!(f, "输入为空"),
        }
    }
}

impl std::error::Error for SubnetError {}

impl From<IpError> for SubnetError {
    fn from(e: IpError) -> Self {
        SubnetError::IpError(e)
    }
}

impl From<MaskError> for SubnetError {
    fn from(e: MaskError) -> Self {
        SubnetError::MaskError(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubnetInfo {
    pub network: u32,
    pub broadcast: u32,
    pub mask: u32,
    pub wildcard: u32,
    pub cidr: u8,
    pub total_ips: u64,
    pub usable_ips: u64,
    pub first_host: u32,
    pub last_host: u32,
    pub ip_class: IpClass,
    pub is_private: bool,
}

pub fn analyze(input: &str) -> Result<SubnetInfo, SubnetError> {
    let normalized = normalize_input(input);
    if normalized.is_empty() {
        return Err(SubnetError::Empty);
    }

    let (ip_addr, cidr) = if let Some((ip_part, mask_part)) = normalized.split_once('/') {
        let ip_addr = ip::parse_ipv4(ip_part)?;
        let cidr = mask::parse_mask(mask_part)?;
        (ip_addr, cidr)
    } else if let Some((ip_part, mask_part)) = normalized.split_once(' ') {
        let ip_addr = ip::parse_ipv4(ip_part)?;
        let cidr = mask::parse_mask(mask_part)?;
        (ip_addr, cidr)
    } else {
        let ip_addr = ip::parse_ipv4(&normalized)?;
        let cidr = match ip::ip_class(ip_addr) {
            IpClass::ClassA => 8,
            IpClass::ClassB => 16,
            IpClass::ClassC => 24,
            IpClass::ClassD | IpClass::ClassE => {
                return Err(SubnetError::NoDefaultMask(ip::ip_class(ip_addr)))
            }
        };
        (ip_addr, cidr)
    };

    let mask = mask::cidr_to_mask(cidr);
    let network = ip_addr & mask;
    let wildcard = mask::wildcard_mask(mask);
    let broadcast = network | wildcard;
    let total_ips: u64 = 1u64 << (32 - cidr as u32);

    let (usable_ips, first_host, last_host) = match cidr {
        31 => (2, network, broadcast),
        32 => (1, network, network),
        _ => (total_ips - 2, network + 1, broadcast - 1),
    };

    Ok(SubnetInfo {
        network,
        broadcast,
        mask,
        wildcard,
        cidr,
        total_ips,
        usable_ips,
        first_host,
        last_host,
        ip_class: ip::ip_class(ip_addr),
        is_private: ip::is_private(ip_addr),
    })
}

pub fn cidr_representation(info: &SubnetInfo) -> String {
    format!("{}/{}", ip::to_dotted(info.network), info.cidr)
}

pub fn mask_binary_grouped(info: &SubnetInfo) -> String {
    ip::to_binary(info.mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_ip_cidr() {
        let info = analyze("192.168.1.100/24").unwrap();
        assert_eq!(info.network, 0xC0A80100);
        assert_eq!(info.broadcast, 0xC0A801FF);
        assert_eq!(info.usable_ips, 254);
        assert_eq!(info.first_host, 0xC0A80101);
        assert_eq!(info.last_host, 0xC0A801FE);
        assert_eq!(info.cidr, 24);
        assert!(info.is_private);
    }

    #[test]
    fn analyze_ip_space_dotted_mask() {
        let info = analyze("10.0.0.1 255.255.255.252").unwrap();
        assert_eq!(info.cidr, 30);
        assert_eq!(info.network, 0x0A000000);
        assert_eq!(info.broadcast, 0x0A000003);
        assert_eq!(info.usable_ips, 2);
        assert_eq!(info.first_host, 0x0A000001);
        assert_eq!(info.last_host, 0x0A000002);
    }

    #[test]
    fn analyze_ip_space_numeric_cidr() {
        let info = analyze("10.0.0.1 30").unwrap();
        assert_eq!(info.cidr, 30);
        assert_eq!(info.network, 0x0A000000);
        assert_eq!(info.broadcast, 0x0A000003);
        assert_eq!(info.usable_ips, 2);
        assert_eq!(info.first_host, 0x0A000001);
        assert_eq!(info.last_host, 0x0A000002);
    }

    #[test]
    fn analyze_rfc3021_31() {
        let info = analyze("10.0.0.0/31").unwrap();
        assert_eq!(info.usable_ips, 2);
        assert_eq!(info.first_host, 0x0A000000);
        assert_eq!(info.last_host, 0x0A000001);
    }

    #[test]
    fn analyze_32() {
        let info = analyze("10.0.0.5/32").unwrap();
        assert_eq!(info.usable_ips, 1);
        assert_eq!(info.first_host, 0x0A000005);
        assert_eq!(info.last_host, 0x0A000005);
    }

    #[test]
    fn analyze_0() {
        let info = analyze("0.0.0.0/0").unwrap();
        assert_eq!(info.total_ips, 4294967296);
        assert_eq!(info.usable_ips, 4294967294);
        assert_eq!(info.first_host, 0x00000001);
        assert_eq!(info.last_host, 0xFFFFFFFE);
    }

    #[test]
    fn analyze_plain_ip_class_c() {
        let info = analyze("192.168.1.1").unwrap();
        assert_eq!(info.cidr, 24);
        assert_eq!(info.network, 0xC0A80100);
    }

    #[test]
    fn analyze_plain_ip_class_a() {
        let info = analyze("10.1.1.1").unwrap();
        assert_eq!(info.cidr, 8);
        assert_eq!(info.network, 0x0A000000);
    }

    #[test]
    fn analyze_plain_ip_class_b() {
        let info = analyze("172.16.1.1").unwrap();
        assert_eq!(info.cidr, 16);
        assert_eq!(info.network, 0xAC100000);
    }

    #[test]
    fn analyze_plain_ip_class_d() {
        assert_eq!(
            analyze("224.0.0.1"),
            Err(SubnetError::NoDefaultMask(IpClass::ClassD))
        );
    }

    #[test]
    fn analyze_plain_ip_class_e() {
        assert_eq!(
            analyze("240.0.0.1"),
            Err(SubnetError::NoDefaultMask(IpClass::ClassE))
        );
    }

    #[test]
    fn analyze_fullwidth() {
        let info = analyze("１９２．１６８．１．１／２４").unwrap();
        assert_eq!(info.network, 0xC0A80100);
        assert_eq!(info.cidr, 24);
    }

    #[test]
    fn analyze_spaces_around_slash() {
        let info = analyze("192.168.1.1 / 24").unwrap();
        assert_eq!(info.network, 0xC0A80100);
        assert_eq!(info.cidr, 24);
    }

    #[test]
    fn cidr_representation_works() {
        let info = analyze("192.168.1.0/24").unwrap();
        assert_eq!(cidr_representation(&info), "192.168.1.0/24");
    }

    #[test]
    fn mask_binary_grouped_works() {
        let info = analyze("192.168.1.1/24").unwrap();
        assert_eq!(
            mask_binary_grouped(&info),
            "11111111.11111111.11111111.00000000"
        );
    }

    #[test]
    fn analyze_invalid_ip() {
        assert!(matches!(
            analyze("999.1.1.1/24"),
            Err(SubnetError::IpError(_))
        ));
    }

    #[test]
    fn analyze_invalid_cidr() {
        assert!(matches!(
            analyze("1.2.3.4/33"),
            Err(SubnetError::MaskError(_))
        ));
    }

    #[test]
    fn analyze_empty() {
        assert_eq!(analyze(""), Err(SubnetError::Empty));
    }
}
