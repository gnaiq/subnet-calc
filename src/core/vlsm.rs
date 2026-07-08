use crate::core::mask;
use crate::core::subnet::{self, SubnetError, SubnetInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VlsmError {
    SubnetError(SubnetError),
    NoRequirements,
}

impl std::fmt::Display for VlsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VlsmError::SubnetError(e) => write!(f, "子网错误: {}", e),
            VlsmError::NoRequirements => write!(f, "需求列表为空"),
        }
    }
}

impl From<SubnetError> for VlsmError {
    fn from(e: SubnetError) -> Self {
        VlsmError::SubnetError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlsmEntry {
    pub name: String,
    pub required_hosts: usize,
    pub allocated_block: u64,
    pub network: u32,
    pub mask: u32,
    pub cidr: u8,
    pub first_host: u32,
    pub last_host: u32,
    pub broadcast: u32,
    pub usable: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlsmFailure {
    pub name: String,
    pub required_hosts: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VlsmResult {
    pub allocated: Vec<VlsmEntry>,
    pub failed: Vec<VlsmFailure>,
}

struct Pending {
    index: usize,
    name: String,
    hosts: usize,
    block_size: u64,
}

struct FailedItem {
    index: usize,
    name: String,
    hosts: usize,
    reason: String,
}

pub fn vlsm(base_input: &str, requirements: Vec<(String, usize)>) -> Result<VlsmResult, VlsmError> {
    let base: SubnetInfo = subnet::analyze(base_input)?;
    if requirements.is_empty() {
        return Err(VlsmError::NoRequirements);
    }

    let mut pending: Vec<Pending> = Vec::new();
    let mut failed: Vec<FailedItem> = Vec::new();

    for (index, (name, hosts)) in requirements.into_iter().enumerate() {
        if hosts == 0 {
            failed.push(FailedItem {
                index,
                name,
                hosts,
                reason: "需求主机数必须 ≥ 1".to_string(),
            });
            continue;
        }
        let needed = match hosts.checked_add(2) {
            Some(v) => v,
            None => {
                failed.push(FailedItem {
                    index,
                    name,
                    hosts,
                    reason: "主机数计算溢出".to_string(),
                });
                continue;
            }
        };
        let block_size = std::cmp::max(needed, 2).next_power_of_two() as u64;
        if block_size > base.total_ips {
            failed.push(FailedItem {
                index,
                name,
                hosts,
                reason: "块大小超过基础网段容量".to_string(),
            });
            continue;
        }
        pending.push(Pending {
            index,
            name,
            hosts,
            block_size,
        });
    }

    pending.sort_by(|a, b| b.block_size.cmp(&a.block_size));

    let mut allocated: Vec<(usize, VlsmEntry)> = Vec::new();
    let mut current_offset = base.network;

    for item in &pending {
        let end = (current_offset as u64) + item.block_size - 1;
        if end > base.broadcast as u64 {
            failed.push(FailedItem {
                index: item.index,
                name: item.name.clone(),
                hosts: item.hosts,
                reason: "地址空间不足".to_string(),
            });
            continue;
        }
        let log2 = item.block_size.trailing_zeros();
        let cidr = (32 - log2) as u8;
        let mask = mask::cidr_to_mask(cidr);
        let network = current_offset;
        let broadcast = network + (item.block_size as u32) - 1;
        let _wildcard = mask::wildcard_mask(mask);
        let (usable, first_host, last_host) = if cidr == 31 {
            (2, network, broadcast)
        } else if cidr == 32 {
            (1, network, network)
        } else {
            (item.block_size - 2, network + 1, broadcast - 1)
        };
        allocated.push((
            item.index,
            VlsmEntry {
                name: item.name.clone(),
                required_hosts: item.hosts,
                allocated_block: item.block_size,
                network,
                mask,
                cidr,
                first_host,
                last_host,
                broadcast,
                usable,
            },
        ));
        current_offset += item.block_size as u32;
    }

    allocated.sort_by_key(|(idx, _)| *idx);
    failed.sort_by_key(|f| f.index);

    let allocated = allocated.into_iter().map(|(_, e)| e).collect();
    let failed = failed
        .into_iter()
        .map(|f| VlsmFailure {
            name: f.name,
            required_hosts: f.hosts,
            reason: f.reason,
        })
        .collect();

    Ok(VlsmResult { allocated, failed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classic() {
        let result = vlsm(
            "192.168.1.0/24",
            vec![
                ("子网1".to_string(), 100),
                ("子网2".to_string(), 50),
                ("子网3".to_string(), 20),
                ("子网4".to_string(), 2),
            ],
        )
        .unwrap();
        assert_eq!(result.allocated.len(), 4);
        assert!(result.failed.is_empty());

        let e1 = &result.allocated[0];
        assert_eq!(e1.name, "子网1");
        assert_eq!(e1.allocated_block, 128);
        assert_eq!(e1.cidr, 25);
        assert_eq!(e1.network, 0xC0A80100);

        let e2 = &result.allocated[1];
        assert_eq!(e2.name, "子网2");
        assert_eq!(e2.allocated_block, 64);
        assert_eq!(e2.cidr, 26);
        assert_eq!(e2.network, 0xC0A80180);

        let e3 = &result.allocated[2];
        assert_eq!(e3.name, "子网3");
        assert_eq!(e3.allocated_block, 32);
        assert_eq!(e3.cidr, 27);
        assert_eq!(e3.network, 0xC0A801C0);

        let e4 = &result.allocated[3];
        assert_eq!(e4.name, "子网4");
        assert_eq!(e4.allocated_block, 4);
        assert_eq!(e4.cidr, 30);
        assert_eq!(e4.network, 0xC0A801E0);
    }

    #[test]
    fn test_order_preserved() {
        let result = vlsm(
            "192.168.1.0/24",
            vec![
                ("A".to_string(), 20),
                ("B".to_string(), 100),
                ("C".to_string(), 50),
                ("D".to_string(), 2),
            ],
        )
        .unwrap();
        assert_eq!(result.allocated.len(), 4);
        assert_eq!(result.allocated[0].name, "A");
        assert_eq!(result.allocated[1].name, "B");
        assert_eq!(result.allocated[2].name, "C");
        assert_eq!(result.allocated[3].name, "D");
    }

    #[test]
    fn test_partial_success() {
        let result = vlsm(
            "192.168.1.0/24",
            vec![("A".to_string(), 200), ("B".to_string(), 100)],
        )
        .unwrap();
        assert_eq!(result.allocated.len(), 1);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].name, "B");
    }

    #[test]
    fn test_zero_hosts() {
        let result = vlsm("192.168.1.0/24", vec![("A".to_string(), 0)]).unwrap();
        assert!(result.allocated.is_empty());
        assert_eq!(result.failed.len(), 1);
        assert!(result.failed[0].reason.contains("≥ 1"));
    }

    #[test]
    fn test_empty_requirements() {
        let result = vlsm("192.168.1.0/24", vec![]);
        assert_eq!(result, Err(VlsmError::NoRequirements));
    }

    #[test]
    fn test_block_exceeds_capacity() {
        let result = vlsm("192.168.1.0/24", vec![("A".to_string(), 300)]).unwrap();
        assert!(result.allocated.is_empty());
        assert_eq!(result.failed.len(), 1);
        assert!(result.failed[0].reason.contains("容量"));
    }
}
