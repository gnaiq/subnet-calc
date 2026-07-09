use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::core::subnet::SubnetInfo;

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 用户原始输入
    pub input: String,
    /// 计算结果
    pub result: SubnetInfo,
}

/// 历史记录存储
pub struct HistoryStore {
    /// 历史条目列表（最新的在前）
    entries: Vec<HistoryEntry>,
    /// 持久化文件路径
    storage_path: PathBuf,
}

impl Default for HistoryStore {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        let storage_path = PathBuf::from(&home)
            .join(".subnet-calc")
            .join("history.json");

        let store = Self {
            entries: Vec::new(),
            storage_path,
        };

        // 尝试加载已有历史
        let mut mutable_store = store;
        mutable_store.load();
        mutable_store
    }
}

impl HistoryStore {
    /// 添加新条目到历史
    pub fn add(&mut self, input: &str, result: SubnetInfo) {
        let entry = HistoryEntry {
            timestamp: Local::now(),
            input: input.to_string(),
            result,
        };

        // 如果已存在相同输入，替换它
        if let Some(pos) = self.entries.iter().position(|e| e.input == entry.input) {
            self.entries.remove(pos);
        }

        // 插入到最前面
        self.entries.insert(0, entry);

        // 自动持久化
        self.save();
    }

    /// 删除指定索引的条目
    pub fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.save();
        }
    }

    /// 清空所有历史
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }

    /// 获取所有条目（只读）
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// 搜索历史（按输入关键词匹配）
    pub fn search(&self, keyword: &str) -> Vec<&HistoryEntry> {
        let kw = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.input.to_lowercase().contains(&kw))
            .collect()
    }

    /// 从文件加载历史
    pub fn load(&mut self) {
        if self.storage_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.storage_path) {
                if let Ok(entries) = serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                    self.entries = entries;
                }
            }
        }
    }

    /// 保存历史到文件
    fn save(&self) {
        if let Some(parent) = self.storage_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directory {:?}: {}", parent, e);
                return;
            }
        }

        if let Ok(content) = serde_json::to_string_pretty(&self.entries) {
            if let Err(e) = fs::write(&self.storage_path, content) {
                eprintln!("Failed to write history to {:?}: {}", self.storage_path, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::subnet;

    fn temp_storage_path() -> PathBuf {
        let tid = std::thread::current().id();
        let tid_str = format!("{:?}", tid).replace(|c: char| !c.is_alphanumeric(), "_");
        std::env::temp_dir().join(format!(
            "subnet_calc_test_{}_{}.json",
            std::process::id(),
            tid_str
        ))
    }

    fn create_test_entry(input: &str) -> HistoryEntry {
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        HistoryEntry {
            timestamp: chrono::Local::now(),
            input: input.to_string(),
            result: info,
        }
    }

    // === HistoryEntry Tests ===

    #[test]
    fn test_history_entry_creation() {
        let entry = create_test_entry("192.168.1.1/24");
        assert_eq!(entry.input, "192.168.1.1/24");
        assert_eq!(entry.result.cidr, 24);
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = create_test_entry("192.168.1.1/24");
        let json = serde_json::to_string(&entry);
        assert!(json.is_ok());
    }

    #[test]
    fn test_history_entry_deserialization() {
        let entry = create_test_entry("192.168.1.1/24");
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.input, entry.input);
    }

    // === HistoryStore Tests ===

    #[test]
    fn test_history_store_default() {
        // Create a fresh store with temp path to avoid loading old data
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path,
        };
        assert!(store.entries().is_empty());
    }

    #[test]
    fn test_history_store_add_entry() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("192.168.1.1/24", info);
        assert_eq!(store.entries().len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_add_multiple_entries() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        for i in 0..5 {
            let info = subnet::analyze("192.168.1.1/24").unwrap();
            store.add(&format!("entry_{}", i), info);
        }

        assert_eq!(store.entries().len(), 5);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_add_duplicate_replaces() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();

        store.add("192.168.1.1/24", info.clone());
        store.add("192.168.1.1/24", info);

        assert_eq!(store.entries().len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_add_inserts_at_front() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info1 = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("first", info1);

        let info2 = subnet::analyze("10.0.0.1/8").unwrap();
        store.add("second", info2);

        assert_eq!(store.entries()[0].input, "second");
        assert_eq!(store.entries()[1].input, "first");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_remove_entry() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        for i in 0..3 {
            let info = subnet::analyze("192.168.1.1/24").unwrap();
            store.add(&format!("entry_{}", i), info);
        }

        assert_eq!(store.entries().len(), 3);
        store.remove(1);
        assert_eq!(store.entries().len(), 2);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_remove_out_of_bounds() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("entry", info);

        store.remove(10); // Should not panic
        assert_eq!(store.entries().len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_clear() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        for i in 0..5 {
            let info = subnet::analyze("192.168.1.1/24").unwrap();
            store.add(&format!("entry_{}", i), info);
        }

        store.clear();
        assert!(store.entries().is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_search_by_input() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("192.168.1.1/24", info);

        let results = store.search("192.168");
        assert_eq!(results.len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_search_no_match() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("192.168.1.1/24", info);

        let results = store.search("nonexistent");
        assert!(results.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_search_case_insensitive() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("TestEntry", info);

        let results = store.search("test");
        assert_eq!(results.len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_search_partial_match() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("192.168.1.1/24", info);

        let results = store.search(".1.");
        assert_eq!(results.len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_save_and_load() {
        let path = temp_storage_path();

        {
            let mut store = HistoryStore {
                entries: Vec::new(),
                storage_path: path.clone(),
            };
            let info = subnet::analyze("192.168.1.1/24").unwrap();
            store.add("192.168.1.1/24", info);

            // add() calls save() internally, so file should exist now
            assert!(path.exists());
        }

        {
            let mut store = HistoryStore {
                entries: Vec::new(),
                storage_path: path.clone(),
            };
            store.load();

            assert_eq!(store.entries().len(), 1);
            assert_eq!(store.entries()[0].input, "192.168.1.1/24");
        }

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_load_nonexistent_file() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        store.load();
        assert!(store.entries().is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_load_invalid_json() {
        let path = temp_storage_path();

        fs::write(&path, "{invalid json}").unwrap();

        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        store.load();

        assert!(store.entries().is_empty());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_load_valid_empty_array() {
        let path = temp_storage_path();

        fs::write(&path, "[]").unwrap();

        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        store.load();

        assert!(store.entries().is_empty());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_load_multiple_entries() {
        let path = temp_storage_path();

        let entries = vec![
            create_test_entry("entry_1"),
            create_test_entry("entry_2"),
            create_test_entry("entry_3"),
        ];

        let json = serde_json::to_string_pretty(&entries).unwrap();
        fs::write(&path, json).unwrap();

        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        store.load();

        assert_eq!(store.entries().len(), 3);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_preserves_order_after_load() {
        let path = temp_storage_path();

        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        for i in 0..5 {
            let info = subnet::analyze("192.168.1.1/24").unwrap();
            store.add(&format!("entry_{}", i), info);
        }

        // Save manually
        if let Ok(content) = serde_json::to_string_pretty(&store.entries) {
            fs::write(&path, content).unwrap();
        }

        // Reload
        let mut loaded_store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        loaded_store.load();

        assert_eq!(loaded_store.entries().len(), 5);
        assert_eq!(loaded_store.entries()[0].input, "entry_4");
        assert_eq!(loaded_store.entries()[4].input, "entry_0");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_directory_creation() {
        let path = temp_storage_path();

        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("test", info);

        assert!(path.exists());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_entries_ref() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("test", info);

        let entries: &[HistoryEntry] = store.entries();
        assert_eq!(entries.len(), 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_search_empty_store() {
        let path = temp_storage_path();
        let store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let results = store.search("anything");
        assert!(results.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_remove_empty_store() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        store.remove(0); // Should not panic
        let _ = fs::remove_file(&path);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_history_store_long_input_string() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();

        let long_input = "a".repeat(1000);
        store.add(&long_input, info);

        assert_eq!(store.entries().len(), 1);
        assert_eq!(store.entries()[0].input.len(), 1000);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_unicode_input() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };
        let info = subnet::analyze("192.168.1.1/24").unwrap();

        store.add("测试_子网", info);

        assert_eq!(store.entries().len(), 1);
        assert_eq!(store.entries()[0].input, "测试_子网");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_history_store_different_cidr_blocks() {
        let path = temp_storage_path();
        let mut store = HistoryStore {
            entries: Vec::new(),
            storage_path: path.clone(),
        };

        let info1 = subnet::analyze("10.0.0.1/8").unwrap();
        store.add("10.0.0.1/8", info1);

        let info2 = subnet::analyze("172.16.0.1/12").unwrap();
        store.add("172.16.0.1/12", info2);

        let info3 = subnet::analyze("192.168.1.1/24").unwrap();
        store.add("192.168.1.1/24", info3);

        assert_eq!(store.entries().len(), 3);
        assert_eq!(store.entries()[0].result.cidr, 24);
        assert_eq!(store.entries()[1].result.cidr, 12);
        assert_eq!(store.entries()[2].result.cidr, 8);
        let _ = fs::remove_file(&path);
    }
}
