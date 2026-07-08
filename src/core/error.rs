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
