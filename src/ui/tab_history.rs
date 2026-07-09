use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::core::export;
use crate::core::history::{HistoryEntry, HistoryStore};
use crate::core::subnet;
use crate::theme;

pub struct HistoryState {
    pub store: HistoryStore,
    pub search_keyword: String,
    pub copied: Option<(String, f64)>,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            store: HistoryStore::default(),
            search_keyword: String::new(),
            copied: None,
        }
    }
}

impl HistoryState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 切换标签页时重新加载历史数据
        self.store.load();
        
        ui.label("历史记录");
        ui.add_space(8.0);

        // 搜索框 + 清空按钮 + 导出按钮
        ui.horizontal(|ui| {
            ui.label("搜索:");
            ui.text_edit_singleline(&mut self.search_keyword);
            if ui.button("清空历史").clicked() {
                self.store.clear();
            }
            ui.separator();
            if ui.button("导出 JSON").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let entries = self.store.entries().to_vec();
                    if let Some(path) = FileDialog::new()
                        .add_filter("JSON Files", &["json"])
                        .set_file_name("history.json")
                        .save_file()
                    {
                        use std::fs;
                        use std::io::Write;
                        let json = export::export_history_json(&entries);
                        if let Ok(mut file) = fs::File::create(&path) {
                            let _ = file.write_all(json.as_bytes());
                        }
                    }
                }
            }
            if ui.button("导出 CSV").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let entries = self.store.entries().to_vec();
                    if let Some(path) = FileDialog::new()
                        .add_filter("CSV Files", &["csv"])
                        .set_file_name("history.csv")
                        .save_file()
                    {
                        use std::fs;
                        use std::io::Write;
                        let csv = export::export_history_csv(&entries);
                        if let Ok(mut file) = fs::File::create(&path) {
                            let _ = file.write_all(csv.as_bytes());
                        }
                    }
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // 获取过滤后的条目
        let filtered: Vec<&HistoryEntry> = if self.search_keyword.trim().is_empty() {
            self.store.entries().iter().collect()
        } else {
            self.store.search(&self.search_keyword).to_vec()
        };

        if filtered.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("暂无历史记录");
            });
            return;
        }

        // 显示历史列表
        egui::ScrollArea::vertical()
            .max_height(f32::MAX)
            .show(ui, |ui| {
                // 获取所有条目的原始索引和引用
                let all_entries = self.store.entries();
                
                // 确定要显示的条目及其原始索引
                let items_to_display: Vec<(usize, &HistoryEntry)> = if self.search_keyword.trim().is_empty() {
                    all_entries.iter().enumerate().map(|(i, e)| (i, e)).collect()
                } else {
                    let kw = self.search_keyword.to_lowercase();
                    all_entries.iter().enumerate()
                        .filter(|(_, e)| e.input.to_lowercase().contains(&kw))
                        .map(|(i, e)| (i, e))
                        .collect()
                };
                
                // 收集需要删除的索引
                let mut to_delete: Vec<usize> = Vec::new();
                
                for (orig_idx, entry) in items_to_display {
                    ui.horizontal(|ui| {
                        // 时间戳
                        let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                        ui.label(time_str);
                        ui.add_space(8.0);

                        // 输入内容
                        ui.label(egui::RichText::new(&entry.input).monospace().strong());
                        ui.add_space(8.0);

                        // CIDR 表示
                        let cidr = subnet::cidr_representation(&entry.result);
                        ui.label(cidr.clone());
                        ui.add_space(8.0);

                        // 可用主机数
                        ui.label(format!("{} 台主机", entry.result.usable_ips));
                        ui.add_space(8.0);

                        // 复制按钮
                        if ui.button("复制").clicked() {
                            ctx.copy_text(cidr.clone());
                            self.copied = Some((cidr, ctx.input(|i| i.time)));
                        }

                        ui.add_space(16.0);

                        // 删除按钮 - 收集待删除索引
                        if ui.button("删除").clicked() {
                            to_delete.push(orig_idx);
                        }
                    });
                    ui.add_space(4.0);
                }
                
                // 在循环结束后执行删除操作
                for idx in to_delete.into_iter().rev() {
                    self.store.remove(idx);
                }
            });

        // 复制提示
        if let Some((text, t)) = &self.copied {
            let now = ctx.input(|i| i.time);
            if now - t < 1.5 {
                ui.colored_label(theme::SUCCESS, format!("已复制: {}", text));
            }
        }
    }
}
