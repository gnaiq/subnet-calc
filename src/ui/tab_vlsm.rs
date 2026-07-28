use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::core::export;
use crate::core::ip;
use crate::core::vlsm::{self, VlsmResult};
use crate::theme;

pub struct VlsmState {
    pub base_input: String,
    pub requirements: Vec<(String, String)>,
    pub result: Option<VlsmResult>,
    pub error: Option<String>,
    pub copied: Option<(String, f64)>,
}

impl Default for VlsmState {
    fn default() -> Self {
        Self {
            base_input: "192.168.1.0/24".to_string(),
            requirements: vec![
                ("子网1".to_string(), "100".to_string()),
                ("子网2".to_string(), "50".to_string()),
                ("子网3".to_string(), "20".to_string()),
                ("子网4".to_string(), "2".to_string()),
            ],
            result: None,
            error: None,
            copied: None,
        }
    }
}

impl VlsmState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical()
            .id_source("vlsm_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.show_inner(ui, ctx);
            });
    }

    fn show_inner(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("基础网段:");
            ui.add(
                egui::TextEdit::singleline(&mut self.base_input)
                    .desired_width(300.0)
                    .font(egui::TextStyle::Monospace),
            );
        });

        ui.add_space(4.0);
        ui.label("主机需求 (名称 / 主机数):");

        let mut to_remove: Option<usize> = None;
        for (i, (name, hosts)) in self.requirements.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(name)
                        .desired_width(120.0)
                        .font(egui::TextStyle::Monospace),
                );
                ui.add(
                    egui::TextEdit::singleline(hosts)
                        .desired_width(80.0)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("删除").clicked() {
                    to_remove = Some(i);
                }
            });
        }
        if let Some(i) = to_remove {
            if self.requirements.len() > 1 {
                self.requirements.remove(i);
            }
        }

        ui.horizontal(|ui| {
            if ui.button("+ 添加需求").clicked() {
                let n = self.requirements.len() + 1;
                self.requirements
                    .push((format!("子网{}", n), "10".to_string()));
            }
            if ui.button("计算").clicked() {
                self.compute();
            }
        });

        if let Some(err) = &self.error {
            ui.colored_label(theme::ERROR, err);
        }

        if let Some(result) = &self.result {
            ui.add_space(8.0);
            if !result.allocated.is_empty() {
                ui.label(format!("已分配子网 ({}):", result.allocated.len()));
                egui::Grid::new("vlsm_result")
                    .num_columns(9)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("名称");
                        ui.label("需求");
                        ui.label("网络地址");
                        ui.label("CIDR");
                        ui.label("掩码");
                        ui.label("范围");
                        ui.label("可用");
                        ui.label("广播");
                        ui.label("");
                        ui.end_row();

                        for e in &result.allocated {
                            ui.label(&e.name);
                            ui.label(e.required_hosts.to_string());
                            ui.label(ip::to_dotted(e.network));
                            ui.label(format!("/{}", e.cidr));
                            ui.label(ip::to_dotted(e.mask));
                            ui.label(format!(
                                "{} ~ {}",
                                ip::to_dotted(e.first_host),
                                ip::to_dotted(e.last_host)
                            ));
                            ui.label(e.usable.to_string());
                            ui.label(ip::to_dotted(e.broadcast));
                            if ui.button("复制").clicked() {
                                let row = format!(
                                    "{},{},{},{},{},{},{},{}",
                                    e.name,
                                    e.required_hosts,
                                    ip::to_dotted(e.network),
                                    e.cidr,
                                    ip::to_dotted(e.mask),
                                    ip::to_dotted(e.first_host),
                                    ip::to_dotted(e.last_host),
                                    ip::to_dotted(e.broadcast)
                                );
                                ctx.copy_text(row.clone());
                                self.copied = Some((row, ctx.input(|i| i.time)));
                            }
                            ui.end_row();
                        }
                    });
            }

            if !result.failed.is_empty() {
                ui.add_space(8.0);
                ui.colored_label(theme::ERROR, format!("失败项 ({}):", result.failed.len()));
                egui::Grid::new("vlsm_failed")
                    .num_columns(3)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("名称");
                        ui.label("需求");
                        ui.label("原因");
                        ui.end_row();
                        for f in &result.failed {
                            ui.colored_label(theme::ERROR, &f.name);
                            ui.label(f.required_hosts.to_string());
                            ui.colored_label(theme::ERROR, &f.reason);
                            ui.end_row();
                        }
                    });
            }
        }

        // 导出按钮区域
        ui.add_space(12.0);
        if let Some(result) = &self.result {
            let result_clone = result.clone();
            ui.horizontal(|ui| {
                ui.label("导出:");
                if ui.button("JSON").clicked() {
                    self.export_json(ctx, &result_clone);
                }
                if ui.button("CSV").clicked() {
                    self.export_csv(ctx, &result_clone);
                }
                if ui.button("Markdown").clicked() {
                    self.export_markdown(ctx, &result_clone);
                }
            });
        }

        if let Some((text, t)) = &self.copied {
            let now = ctx.input(|i| i.time);
            if now - t < 1.5 {
                ui.colored_label(theme::SUCCESS, format!("已复制: {}", text));
            }
        }
    }

    fn export_json(&mut self, _ctx: &egui::Context, result: &VlsmResult) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let json = export::export_vlsm_json(result);
            if let Some(path) = FileDialog::new()
                .add_filter("JSON Files", &["json"])
                .set_file_name("vlsm_result.json")
                .save_file()
            {
                use std::fs;
                use std::io::Write;
                if let Ok(mut file) = fs::File::create(&path) {
                    let _ = file.write_all(json.as_bytes());
                }
            }
        }
    }

    fn export_csv(&mut self, _ctx: &egui::Context, result: &VlsmResult) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let csv = export::export_vlsm_csv(result);
            if let Some(path) = FileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .set_file_name("vlsm_result.csv")
                .save_file()
            {
                use std::fs;
                use std::io::Write;
                if let Ok(mut file) = fs::File::create(&path) {
                    let _ = file.write_all(csv.as_bytes());
                }
            }
        }
    }

    fn export_markdown(&mut self, _ctx: &egui::Context, result: &VlsmResult) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let md = export::export_vlsm_markdown(result);
            if let Some(path) = FileDialog::new()
                .add_filter("Markdown Files", &["md"])
                .set_file_name("vlsm_result.md")
                .save_file()
            {
                use std::fs;
                use std::io::Write;
                if let Ok(mut file) = fs::File::create(&path) {
                    let _ = file.write_all(md.as_bytes());
                }
            }
        }
    }

    fn compute(&mut self) {
        let reqs: Vec<(String, usize)> = self
            .requirements
            .iter()
            .filter_map(|(name, hosts)| {
                let h: usize = hosts.trim().parse().ok()?;
                Some((name.clone(), h))
            })
            .collect();

        match vlsm::vlsm(&self.base_input, reqs) {
            Ok(r) => {
                self.result = Some(r);
                self.error = None;
            }
            Err(e) => {
                self.result = None;
                self.error = Some(e.to_string());
            }
        }
    }
}
