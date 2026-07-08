use crate::core::ip;
use crate::core::mask;
use crate::core::subnet::{self, SubnetInfo};

pub fn contains(a: &SubnetInfo, b: &SubnetInfo) -> bool {
    a.cidr <= b.cidr && (b.network & a.mask) == a.network
}

pub fn ip_in_subnet(ip: u32, net: &SubnetInfo) -> bool {
    (ip & net.mask) == net.network
}

pub fn overlaps(a: &SubnetInfo, b: &SubnetInfo) -> bool {
    let start = a.network.max(b.network);
    let end = a.broadcast.min(b.broadcast);
    start <= end
}

pub fn can_aggregate(a: &SubnetInfo, b: &SubnetInfo) -> Option<SubnetInfo> {
    if a.cidr != b.cidr {
        return None;
    }
    if a.cidr == 0 {
        return None;
    }
    let new_cidr = a.cidr - 1;
    let new_mask = mask::cidr_to_mask(new_cidr);
    if (a.network & new_mask) != (b.network & new_mask) {
        return None;
    }
    if a.network == b.network {
        return None;
    }
    let new_network = a.network & new_mask;
    let input = format!("{}/{}", ip::to_dotted(new_network), new_cidr);
    subnet::analyze(&input).ok()
}

pub fn aggregate_many(nets: Vec<SubnetInfo>) -> Vec<SubnetInfo> {
    let mut result: Vec<SubnetInfo> = nets;
    result.sort_by(|x, y| x.network.cmp(&y.network).then(x.cidr.cmp(&y.cidr)));
    loop {
        let mut merged = false;
        let mut next: Vec<SubnetInfo> = Vec::new();
        let mut i = 0;
        while i < result.len() {
            if i + 1 < result.len() {
                if let Some(aggregated) = can_aggregate(&result[i], &result[i + 1]) {
                    next.push(aggregated);
                    merged = true;
                    i += 2;
                    continue;
                }
            }
            next.push(result[i]);
            i += 1;
        }
        result = next;
        if !merged {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::subnet::analyze;

    fn s(input: &str) -> SubnetInfo {
        analyze(input).unwrap()
    }

    #[test]
    fn contains_works() {
        assert!(contains(&s("192.168.0.0/16"), &s("192.168.1.0/24")));
        assert!(!contains(&s("192.168.1.0/24"), &s("192.168.0.0/16")));
        assert!(contains(&s("192.168.1.0/24"), &s("192.168.1.0/24")));
        assert!(!contains(&s("10.0.0.0/8"), &s("192.168.0.0/16")));
    }

    #[test]
    fn ip_in_subnet_works() {
        let net = s("192.168.1.0/24");
        assert!(ip_in_subnet(0xC0A80105, &net));
        assert!(!ip_in_subnet(0xC0A80205, &net));
        assert!(ip_in_subnet(0xC0A801FF, &net));
    }

    #[test]
    fn overlaps_works() {
        assert!(overlaps(&s("192.168.1.0/24"), &s("192.168.1.128/25")));
        assert!(!overlaps(&s("192.168.1.0/24"), &s("192.168.2.0/24")));
        assert!(!overlaps(&s("192.168.1.0/25"), &s("192.168.1.128/25")));
        assert!(overlaps(&s("192.168.0.0/16"), &s("192.168.1.0/24")));
    }

    #[test]
    fn can_aggregate_works() {
        assert_eq!(
            can_aggregate(&s("192.168.0.0/24"), &s("192.168.1.0/24")),
            Some(s("192.168.0.0/23"))
        );
        assert_eq!(
            can_aggregate(&s("192.168.0.0/24"), &s("192.168.2.0/24")),
            None
        );
        assert_eq!(
            can_aggregate(&s("192.168.1.0/24"), &s("192.168.2.0/24")),
            None
        );
        assert_eq!(
            can_aggregate(&s("192.168.0.0/24"), &s("192.168.0.0/24")),
            None
        );
        assert_eq!(can_aggregate(&s("0.0.0.0/0"), &s("192.168.0.0/24")), None);
        assert_eq!(
            can_aggregate(&s("10.0.0.0/25"), &s("10.0.0.128/25")),
            Some(s("10.0.0.0/24"))
        );
    }

    #[test]
    fn aggregate_many_works() {
        assert_eq!(
            aggregate_many(vec![
                s("192.168.0.0/24"),
                s("192.168.1.0/24"),
                s("192.168.2.0/24"),
                s("192.168.3.0/24"),
            ]),
            vec![s("192.168.0.0/22")]
        );
        assert_eq!(
            aggregate_many(vec![s("10.0.0.0/24"), s("10.0.2.0/24")]),
            vec![s("10.0.0.0/24"), s("10.0.2.0/24")]
        );
        assert_eq!(
            aggregate_many(vec![s("192.168.0.0/24"), s("192.168.1.0/24")]),
            vec![s("192.168.0.0/23")]
        );
        assert_eq!(
            aggregate_many(vec![
                s("192.168.0.0/24"),
                s("192.168.2.0/24"),
                s("192.168.1.0/24"),
                s("192.168.3.0/24"),
            ]),
            vec![s("192.168.0.0/22")]
        );
    }
}
