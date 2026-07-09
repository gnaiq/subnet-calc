use crate::core::history::HistoryEntry;
use crate::core::subnet::{self, SubnetInfo};
use crate::core::vlsm::VlsmResult;

/// 导出格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// Markdown 格式
    Markdown,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "JSON"),
            ExportFormat::Csv => write!(f, "CSV"),
            ExportFormat::Markdown => write!(f, "Markdown"),
        }
    }
}

/// 将子网信息导出为 JSON 字符串（单个对象）
pub fn export_json(info: &SubnetInfo) -> String {
    let json = serde_json::json!({
        "network": info.network,
        "broadcast": info.broadcast,
        "mask": info.mask,
        "wildcard": info.wildcard,
        "cidr": info.cidr,
        "total_ips": info.total_ips,
        "usable_ips": info.usable_ips,
        "first_host": info.first_host,
        "last_host": info.last_host,
        "ip_class": format!("{:?}", info.ip_class),
        "is_private": info.is_private,
    });
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

/// 将多个子网信息导出为 JSON 数组
pub fn export_json_array(infos: &[SubnetInfo]) -> String {
    let array: Vec<serde_json::Value> = infos.iter().map(|info| {
        serde_json::json!({
            "network": to_dotted(info.network),
            "broadcast": to_dotted(info.broadcast),
            "mask": to_dotted(info.mask),
            "wildcard": to_dotted(info.wildcard),
            "cidr": cidr_representation(info),
            "total_ips": info.total_ips,
            "usable_ips": info.usable_ips,
            "first_host": to_dotted(info.first_host),
            "last_host": to_dotted(info.last_host),
            "ip_class": info.ip_class.description(),
            "is_private": info.is_private,
        })
    }).collect();
    serde_json::to_string_pretty(&array).unwrap_or_default()
}

/// 将子网信息导出为 CSV 行（无表头，用于单条）
pub fn export_csv_line(info: &SubnetInfo) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{}",
        cidr_representation(info),
        to_dotted(info.network),
        to_dotted(info.broadcast),
        to_dotted(info.mask),
        to_dotted(info.wildcard),
        info.total_ips,
        info.usable_ips,
        to_dotted(info.first_host),
        to_dotted(info.last_host),
    )
}

/// 将子网信息导出为带表头的 CSV
pub fn export_csv_with_header(infos: &[SubnetInfo]) -> String {
    let mut output = String::from("CIDR,网络地址,广播地址,子网掩码,反掩码,IP总数,可用主机数,首个可用IP,末个可用IP\n");
    for info in infos {
        output.push_str(&export_csv_line(info));
        output.push('\n');
    }
    output
}

/// 将子网信息导出为 Markdown 表格行（无表头，用于单条）
pub fn export_markdown_row(info: &SubnetInfo) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        cidr_representation(info),
        to_dotted(info.network),
        to_dotted(info.broadcast),
        to_dotted(info.mask),
        to_dotted(info.wildcard),
        info.total_ips,
        info.usable_ips,
        to_dotted(info.first_host),
        to_dotted(info.last_host),
    )
}

/// 将子网信息导出为带表头的 Markdown 表格
pub fn export_markdown_table(infos: &[SubnetInfo]) -> String {
    let mut output = String::from("| CIDR | 网络地址 | 广播地址 | 子网掩码 | 反掩码 | IP总数 | 可用主机数 | 首个可用IP | 末个可用IP |\n");
    output.push_str("|------|----------|----------|----------|--------|--------|------------|------------|------------|\n");
    for info in infos {
        output.push_str(&export_markdown_row(info));
        output.push('\n');
    }
    output
}

/// 辅助函数：将 u32 IP 转换为点分十进制
fn to_dotted(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF,
    )
}

/// 辅助函数：CIDR 表示
fn cidr_representation(info: &SubnetInfo) -> String {
    subnet::cidr_representation(info)
}

/// 将历史记录导出为 JSON 数组（包含时间戳和输入）
pub fn export_history_json(entries: &[HistoryEntry]) -> String {
    let array: Vec<serde_json::Value> = entries.iter().map(|entry| {
        serde_json::json!({
            "timestamp": entry.timestamp.to_rfc3339(),
            "input": entry.input,
            "network": to_dotted(entry.result.network),
            "broadcast": to_dotted(entry.result.broadcast),
            "mask": to_dotted(entry.result.mask),
            "wildcard": to_dotted(entry.result.wildcard),
            "cidr": cidr_representation(&entry.result),
            "total_ips": entry.result.total_ips,
            "usable_ips": entry.result.usable_ips,
            "first_host": to_dotted(entry.result.first_host),
            "last_host": to_dotted(entry.result.last_host),
            "ip_class": entry.result.ip_class.description(),
            "is_private": entry.result.is_private,
        })
    }).collect();
    serde_json::to_string_pretty(&array).unwrap_or_default()
}

/// 将历史记录导出为带表头的 CSV
pub fn export_history_csv(entries: &[HistoryEntry]) -> String {
    let mut output = String::from("时间戳,输入,CIDR,网络地址,广播地址,子网掩码,反掩码,IP总数,可用主机数,首个可用IP,末个可用IP\n");
    for entry in entries {
        output.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.input,
            cidr_representation(&entry.result),
            to_dotted(entry.result.network),
            to_dotted(entry.result.broadcast),
            to_dotted(entry.result.mask),
            to_dotted(entry.result.wildcard),
            entry.result.total_ips,
            entry.result.usable_ips,
            to_dotted(entry.result.first_host),
            to_dotted(entry.result.last_host),
        ));
        output.push('\n');
    }
    output
}

/// 将 VLSM 结果导出为 JSON 字符串
pub fn export_vlsm_json(result: &VlsmResult) -> String {
    let json = serde_json::json!({
        "allocated": result.allocated.iter().map(|e| {
            serde_json::json!({
                "name": e.name,
                "required_hosts": e.required_hosts,
                "block_size": e.allocated_block,
                "network": to_dotted(e.network),
                "mask": to_dotted(e.mask),
                "cidr": format!("/{}", e.cidr),
                "first_host": to_dotted(e.first_host),
                "last_host": to_dotted(e.last_host),
                "broadcast": to_dotted(e.broadcast),
                "usable": e.usable,
            })
        }).collect::<Vec<_>>(),
        "failed": result.failed.iter().map(|f| {
            serde_json::json!({
                "name": f.name,
                "required_hosts": f.required_hosts,
                "reason": f.reason,
            })
        }).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

/// 将 VLSM 结果导出为带表头的 CSV
pub fn export_vlsm_csv(result: &VlsmResult) -> String {
    let mut output = String::from("名称,需求主机数,块大小,网络地址,CIDR,掩码,首个可用IP,末个可用IP,广播地址,可用主机数\n");
    for e in &result.allocated {
        output.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            e.name,
            e.required_hosts,
            e.allocated_block,
            to_dotted(e.network),
            format!("/{}", e.cidr),
            to_dotted(e.mask),
            to_dotted(e.first_host),
            to_dotted(e.last_host),
            to_dotted(e.broadcast),
            e.usable,
        ));
    }
    if !result.failed.is_empty() {
        output.push_str("\n失败项:\n");
        output.push_str("名称,需求主机数,原因\n");
        for f in &result.failed {
            output.push_str(&format!("{},{},{}\n", f.name, f.required_hosts, f.reason));
        }
    }
    output
}

/// 将 VLSM 结果导出为 Markdown 表格
pub fn export_vlsm_markdown(result: &VlsmResult) -> String {
    let mut output = String::from("| 名称 | 需求主机数 | 块大小 | 网络地址 | CIDR | 掩码 | 首个可用IP | 末个可用IP | 广播地址 | 可用主机数 |\n");
    output.push_str("|------|------------|--------|----------|------|------|------------|------------|----------|------------|\n");
    for e in &result.allocated {
        output.push_str(&format!(
            "| {} | {} | {} | {} | /{} | {} | {} | {} | {} | {} |\n",
            e.name,
            e.required_hosts,
            e.allocated_block,
            to_dotted(e.network),
            e.cidr,
            to_dotted(e.mask),
            to_dotted(e.first_host),
            to_dotted(e.last_host),
            to_dotted(e.broadcast),
            e.usable,
        ));
    }
    if !result.failed.is_empty() {
        output.push_str("\n**失败项:**\n\n");
        output.push_str("| 名称 | 需求主机数 | 原因 |\n");
        output.push_str("|------|------------|------|\n");
        for f in &result.failed {
            output.push_str(&format!("| {} | {} | {} |\n", f.name, f.required_hosts, f.reason));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::subnet;
    use crate::core::vlsm;

    fn sample_subnet_info() -> SubnetInfo {
        subnet::analyze("192.168.1.100/24").unwrap()
    }

    // === ExportFormat Display ===

    #[test]
    fn export_format_display_json() {
        assert_eq!(format!("{}", ExportFormat::Json), "JSON");
    }

    #[test]
    fn export_format_display_csv() {
        assert_eq!(format!("{}", ExportFormat::Csv), "CSV");
    }

    #[test]
    fn export_format_display_markdown() {
        assert_eq!(format!("{}", ExportFormat::Markdown), "Markdown");
    }

    // === Single Subnet Export ===

    #[test]
    fn export_json_contains_required_fields() {
        let info = sample_subnet_info();
        let json = export_json(&info);
        assert!(json.contains("\"network\""));
        assert!(json.contains("\"broadcast\""));
        assert!(json.contains("\"mask\""));
        assert!(json.contains("\"cidr\""));
        assert!(json.contains("\"usable_ips\""));
    }

    #[test]
    fn export_json_valid_json_string() {
        let info = sample_subnet_info();
        let json = export_json(&info);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Exported JSON should be valid: {}", json);
    }

    #[test]
    fn export_csv_line_format() {
        let info = sample_subnet_info();
        let line = export_csv_line(&info);
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields.len(), 9, "CSV line should have 9 fields");
        assert_eq!(fields[0], "192.168.1.0/24");
    }

    #[test]
    fn export_csv_with_header_has_header() {
        let info = sample_subnet_info();
        let csv = export_csv_with_header(&[info]);
        assert!(csv.starts_with("CIDR,网络地址,广播地址,子网掩码,反掩码,IP总数,可用主机数,首个可用IP,末个可用IP\n"));
    }

    #[test]
    fn export_csv_with_header_multiple_entries() {
        let info1 = subnet::analyze("192.168.1.1/24").unwrap();
        let info2 = subnet::analyze("10.0.0.1/16").unwrap();
        let csv = export_csv_with_header(&[info1, info2]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3, "Should have header + 2 data rows");
        assert!(lines[1].starts_with("192.168.1.0/24,"));
        assert!(lines[2].starts_with("10.0.0.0/16,"));
    }

    #[test]
    fn export_csv_with_header_empty_input() {
        let csv = export_csv_with_header(&[]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1, "Empty input should only have header");
    }

    #[test]
    fn export_markdown_row_format() {
        let info = sample_subnet_info();
        let row = export_markdown_row(&info);
        assert!(row.starts_with("| "));
        assert!(row.ends_with(" |"));
    }

    #[test]
    fn export_markdown_table_has_header_and_separator() {
        let info = sample_subnet_info();
        let table = export_markdown_table(&[info]);
        let lines: Vec<&str> = table.lines().collect();
        assert!(lines[0].contains("CIDR"));
        assert!(lines[1].contains("------"));
    }

    #[test]
    fn export_markdown_table_multiple_entries() {
        let info1 = subnet::analyze("192.168.1.1/24").unwrap();
        let info2 = subnet::analyze("10.0.0.1/8").unwrap();
        let table = export_markdown_table(&[info1, info2]);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 4, "Should have header + separator + 2 data rows");
    }

    // === Batch Export Functions ===

    #[test]
    fn export_json_array_single_entry() {
        let info = sample_subnet_info();
        let json = export_json_array(&[info]);
        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().len(), 1);
    }

    #[test]
    fn export_json_array_multiple_entries() {
        let info1 = subnet::analyze("192.168.1.1/24").unwrap();
        let info2 = subnet::analyze("10.0.0.1/16").unwrap();
        let json = export_json_array(&[info1, info2]);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["cidr"], "192.168.1.0/24");
        assert_eq!(parsed[1]["cidr"], "10.0.0.0/16");
    }

    #[test]
    fn export_json_array_empty_input() {
        let json = export_json_array(&[]);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn export_json_array_uses_dotted_notation() {
        let info = sample_subnet_info();
        let json = export_json_array(&[info]);
        assert!(json.contains("192.168.1.0"));
        assert!(!json.contains("0xC0A80100"));
    }

    // === History Export ===

    #[test]
    fn export_history_json_structure() {
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        let entry = HistoryEntry {
            timestamp: chrono::Local::now(),
            input: "192.168.1.1/24".to_string(),
            result: info,
        };
        let json = export_history_json(&[entry]);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].get("timestamp").is_some());
        assert!(parsed[0].get("input").is_some());
        assert!(parsed[0].get("network").is_some());
    }

    #[test]
    fn export_history_json_empty_input() {
        let json = export_history_json(&[]);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn export_history_csv_has_header() {
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        let entry = HistoryEntry {
            timestamp: chrono::Local::now(),
            input: "192.168.1.1/24".to_string(),
            result: info,
        };
        let csv = export_history_csv(&[entry]);
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines[0].starts_with("时间戳,输入,CIDR,"));
    }

    #[test]
    fn export_history_csv_empty_input() {
        let csv = export_history_csv(&[]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1, "Should only have header");
    }

    // === VLSM Export ===

    #[test]
    fn export_vlsm_json_allocated_entries() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![
                ("子网1".to_string(), 100),
                ("子网2".to_string(), 50),
            ],
        ).unwrap();
        let json = export_vlsm_json(&result);
        assert!(json.contains("\"allocated\""));
        assert!(json.contains("\"子网1\""));
        assert!(json.contains("\"子网2\""));
    }

    #[test]
    fn export_vlsm_json_failed_entries() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![
                ("A".to_string(), 300),
            ],
        ).unwrap();
        let json = export_vlsm_json(&result);
        assert!(json.contains("\"failed\""));
        assert!(json.contains("\"A\""));
    }

    #[test]
    fn export_vlsm_json_empty_result() {
        let empty_result = VlsmResult {
            allocated: vec![],
            failed: vec![],
        };
        let json = export_vlsm_json(&empty_result);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["allocated"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["failed"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn export_vlsm_csv_allocated_section() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![("子网1".to_string(), 10)],
        ).unwrap();
        let csv = export_vlsm_csv(&result);
        assert!(csv.starts_with("名称,需求主机数,块大小,网络地址,CIDR,掩码,首个可用IP,末个可用IP,广播地址,可用主机数\n"));
        assert!(csv.contains("子网1"));
    }

    #[test]
    fn export_vlsm_csv_failed_section() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![("A".to_string(), 300)],
        ).unwrap();
        let csv = export_vlsm_csv(&result);
        assert!(csv.contains("失败项:"));
        assert!(csv.contains("名称,需求主机数,原因"));
    }

    #[test]
    fn export_vlsm_csv_empty_result() {
        let empty_result = VlsmResult {
            allocated: vec![],
            failed: vec![],
        };
        let csv = export_vlsm_csv(&empty_result);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1, "Should only have header");
    }

    #[test]
    fn export_vlsm_markdown_allocated_section() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![("子网1".to_string(), 10)],
        ).unwrap();
        let md = export_vlsm_markdown(&result);
        assert!(md.starts_with("| 名称 | 需求主机数 | 块大小 |"));
        assert!(md.contains("子网1"));
    }

    #[test]
    fn export_vlsm_markdown_failed_section() {
        let result = vlsm::vlsm(
            "192.168.1.0/24",
            vec![("A".to_string(), 300)],
        ).unwrap();
        let md = export_vlsm_markdown(&result);
        assert!(md.contains("**失败项:**"));
    }

    #[test]
    fn export_vlsm_markdown_empty_result() {
        let empty_result = VlsmResult {
            allocated: vec![],
            failed: vec![],
        };
        let md = export_vlsm_markdown(&empty_result);
        let lines: Vec<&str> = md.lines().collect();
        assert_eq!(lines.len(), 2, "Should have header + separator");
    }

    // === Helper Function Tests ===

    #[test]
    fn to_dotted_converts_correctly() {
        // 192.168.1.0 = 0xC0A80100
        assert_eq!(to_dotted(0xC0A80100), "192.168.1.0");
    }

    #[test]
    fn to_dotted_zero() {
        assert_eq!(to_dotted(0x00000000), "0.0.0.0");
    }

    #[test]
    fn to_dotted_broadcast() {
        assert_eq!(to_dotted(0xFFFFFFFF), "255.255.255.255");
    }
}
