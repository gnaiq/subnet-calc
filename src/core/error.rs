use std::fmt;

use crate::core::ip::IpError;
use crate::core::mask::MaskError;
use crate::core::subnet::SubnetError;
use crate::core::vlsm::VlsmError;

#[derive(Debug)]
pub enum CoreError {
    Ip(IpError),
    Mask(MaskError),
    Subnet(SubnetError),
    Vlsm(VlsmError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Ip(e) => write!(f, "{}", e),
            CoreError::Mask(e) => write!(f, "{}", e),
            CoreError::Subnet(e) => write!(f, "{}", e),
            CoreError::Vlsm(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<IpError> for CoreError {
    fn from(e: IpError) -> Self {
        CoreError::Ip(e)
    }
}

impl From<MaskError> for CoreError {
    fn from(e: MaskError) -> Self {
        CoreError::Mask(e)
    }
}

impl From<SubnetError> for CoreError {
    fn from(e: SubnetError) -> Self {
        CoreError::Subnet(e)
    }
}

impl From<VlsmError> for CoreError {
    fn from(e: VlsmError) -> Self {
        CoreError::Vlsm(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ip;
    use crate::core::subnet;
    use crate::core::vlsm;

    // === CoreError Display Tests ===

    #[test]
    fn test_core_error_display_ip() {
        let result = ip::parse_ipv4("999.999.999.999");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let core_err = CoreError::Ip(err);
        let display = format!("{}", core_err);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_core_error_display_mask() {
        // 1 is an invalid mask (not contiguous 1-bits from MSB)
        // Valid masks: 0, 0xFFFFFFFF, 0xFFFFFF00 (/24), etc.
        // Invalid: 1 (0x00000001)
        let mask_err = MaskError::InvalidMask(1);
        let core_err = CoreError::Mask(mask_err);
        let display = format!("{}", core_err);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_core_error_display_subnet() {
        let result = subnet::analyze("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let core_err = CoreError::Subnet(err);
        let display = format!("{}", core_err);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_core_error_display_vlsm() {
        // vlsm() returns Ok even with invalid requirements (0 hosts),
        // it puts failures in the failed list. Use invalid network instead.
        let result = vlsm::vlsm("invalid", vec![("A".to_string(), 10)]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let core_err = CoreError::Vlsm(err);
        let display = format!("{}", core_err);
        assert!(!display.is_empty());
    }

    // === From Trait Tests ===

    #[test]
    fn test_from_ip_error() {
        let ip_err = ip::parse_ipv4("999.999.999.999").unwrap_err();
        let core_err: CoreError = ip_err.into();
        match core_err {
            CoreError::Ip(_) => {}
            _ => panic!("Expected CoreError::Ip"),
        }
    }

    #[test]
    fn test_from_mask_error() {
        // 1 is an invalid mask (not contiguous 1-bits from MSB)
        let mask_err = MaskError::InvalidMask(1);
        let core_err: CoreError = mask_err.into();
        match core_err {
            CoreError::Mask(_) => {}
            _ => panic!("Expected CoreError::Mask"),
        }
    }

    #[test]
    fn test_from_subnet_error() {
        let subnet_err = subnet::analyze("invalid").unwrap_err();
        let core_err: CoreError = subnet_err.into();
        match core_err {
            CoreError::Subnet(_) => {}
            _ => panic!("Expected CoreError::Subnet"),
        }
    }

    #[test]
    fn test_from_vlsm_error() {
        // Use invalid network string to trigger actual VlsmError
        let vlsm_err = vlsm::vlsm("invalid", vec![("A".to_string(), 10)]).unwrap_err();
        let core_err: CoreError = vlsm_err.into();
        match core_err {
            CoreError::Vlsm(_) => {}
            _ => panic!("Expected CoreError::Vlsm"),
        }
    }

    // === Debug Trait Tests ===

    #[test]
    fn test_core_error_debug_ip() {
        let ip_err = ip::parse_ipv4("999.999.999.999").unwrap_err();
        let core_err = CoreError::Ip(ip_err);
        let debug_str = format!("{:?}", core_err);
        assert!(debug_str.contains("Ip"));
    }

    #[test]
    fn test_core_error_debug_mask() {
        let mask_err = MaskError::InvalidMask(1);
        let core_err = CoreError::Mask(mask_err);
        let debug_str = format!("{:?}", core_err);
        assert!(debug_str.contains("Mask"));
    }

    #[test]
    fn test_core_error_debug_subnet() {
        let subnet_err = subnet::analyze("invalid").unwrap_err();
        let core_err = CoreError::Subnet(subnet_err);
        let debug_str = format!("{:?}", core_err);
        assert!(debug_str.contains("Subnet"));
    }

    #[test]
    fn test_core_error_debug_vlsm() {
        let vlsm_err = vlsm::vlsm("invalid", vec![("A".to_string(), 10)]).unwrap_err();
        let core_err = CoreError::Vlsm(vlsm_err);
        let debug_str = format!("{:?}", core_err);
        assert!(debug_str.contains("Vlsm"));
    }

    // === Error Trait Tests ===

    #[test]
    fn test_core_error_is_std_error() {
        let subnet_err = subnet::analyze("invalid").unwrap_err();
        let core_err = CoreError::Subnet(subnet_err);
        let std_err: &dyn std::error::Error = &core_err;
        assert!(!std_err.to_string().is_empty());
    }

    // === Integration Tests ===

    #[test]
    fn test_core_error_chain_ip_to_subnet() {
        // Test that IP errors can propagate through subnet analysis
        let result = subnet::analyze("invalid.ip.address/24");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let core_err = CoreError::Subnet(err);
        assert!(matches!(core_err, CoreError::Subnet(_)));
    }

    #[test]
    fn test_core_error_multiple_variants() {
        let ip_err = ip::parse_ipv4("999.999.999.999").unwrap_err();
        let mask_err = MaskError::InvalidMask(0);
        let subnet_err = subnet::analyze("invalid").unwrap_err();
        let vlsm_err = vlsm::vlsm("invalid", vec![("A".to_string(), 10)]).unwrap_err();

        let core_errs = vec![
            CoreError::Ip(ip_err),
            CoreError::Mask(mask_err),
            CoreError::Subnet(subnet_err),
            CoreError::Vlsm(vlsm_err),
        ];

        assert_eq!(core_errs.len(), 4);

        // Verify each variant displays correctly
        for err in &core_errs {
            let display = format!("{}", err);
            assert!(!display.is_empty(), "Display should not be empty");
        }
    }

    #[test]
    fn test_core_error_conversion_consistency() {
        // Test that Into/From conversions are consistent
        let ip_err = ip::parse_ipv4("999.999.999.999").unwrap_err();

        // Using Into trait
        let core_err: CoreError = ip_err.clone().into();
        assert!(matches!(core_err, CoreError::Ip(_)));

        // Using From trait explicitly
        let core_err2 = CoreError::Ip(ip_err);
        assert!(matches!(core_err2, CoreError::Ip(_)));
    }
}
