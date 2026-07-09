use eframe::egui;
#[cfg(target_arch = "wasm32")]
use eframe::egui::Context;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::core::export;
use crate::core::history::HistoryStore;
use crate::core::ip;
use crate::core::subnet::{self, mask_binary_grouped, SubnetInfo};
use crate::theme;

pub struct BasicState {
    pub input: String,
    pub batch_mode: bool,
    pub batch_input: String,
    pub results: Vec<ResultItem>,
    pub result: Option<SubnetInfo>,
    pub error: Option<String>,
    pub copied: Option<(String, f64)>,
    pub history: HistoryStore,
    pub clipboard_msg: Option<String>,
    pub clipboard_msg_time: f64,
}

struct ResultItem {
    pub input_text: String,
    pub info: SubnetInfo,
    pub error: Option<String>,
}

impl Default for BasicState {
    fn default() -> Self {
        Self {
            input: "192.168.1.1/24".to_string(),
            batch_mode: false,
            batch_input: String::new(),
            results: Vec::new(),
            result: None,
            error: None,
            copied: None,
            history: HistoryStore::default(),
            clipboard_msg: None,
            clipboard_msg_time: 0.0,
        }
    }
}

impl BasicState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 批量模式切换
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.batch_mode, "批量模式");
        });
        ui.add_space(4.0);

        if self.batch_mode {
            // ===== 批量模式 =====
            ui.label("每行一个 IP/CIDR 或 IP 掩码:");
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.batch_input)
                    .min_size(egui::vec2(ui.available_width(), 200.0))
                    .font(egui::TextStyle::Monospace),
            );
            if ui.button("计算全部").clicked() || resp.changed() {
                self.batch_compute();
            }

            // 显示批量结果
            if !self.results.is_empty() {
                ui.add_space(12.0);
                ui.label(format!("共 {} 条结果:", self.results.len()));
                
                for (i, item) in self.results.iter().enumerate() {
                    let input_text = item.input_text.clone();
                    let error = item.error.clone();
                    let info = item.info.clone();
                    
                    ui.collapsing(format!("{}. {}", i + 1, &input_text), |ui| {
                        let ctx_local = ctx.clone();
                        if let Some(ref err) = error {
                            ui.colored_label(theme::ERROR, err);
                        } else {
                            egui::Grid::new(format!("batch_grid_{}", i))
                                .num_columns(3)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("网络地址");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.network)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("广播地址");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.broadcast)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("子网掩码");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.mask)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("反掩码");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.wildcard)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("CIDR 表示");
                                    ui.label(egui::RichText::new(&subnet::cidr_representation(&info)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("IP 类别");
                                    ui.label(egui::RichText::new(info.ip_class.description()).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("是否私有");
                                    ui.label(egui::RichText::new(if info.is_private { "是" } else { "否" }).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("IP 总数");
                                    ui.label(egui::RichText::new(&info.total_ips.to_string()).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("可用主机数");
                                    ui.label(egui::RichText::new(&info.usable_ips.to_string()).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("首个可用 IP");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.first_host)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                    ui.label("末个可用 IP");
                                    ui.label(egui::RichText::new(&ip::to_dotted(info.last_host)).monospace());
                                    ui.label("");
                                    ui.end_row();
                                });
                            
                            // 导出按钮
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.small_button("复制全部").clicked() {
                                    let all_text = format!(
                                        "网络地址: {}\n广播地址: {}\n子网掩码: {}\n反掩码: {}\nCIDR 表示: {}\nIP 类别: {}\n是否私有: {}\nIP 总数: {}\n可用主机数: {}\n首个可用 IP: {}\n末个可用 IP: {}",
                                        ip::to_dotted(info.network),
                                        ip::to_dotted(info.broadcast),
                                        ip::to_dotted(info.mask),
                                        ip::to_dotted(info.wildcard),
                                        subnet::cidr_representation(&info),
                                        info.ip_class.description(),
                                        if info.is_private { "是" } else { "否" },
                                        info.total_ips,
                                        info.usable_ips,
                                        ip::to_dotted(info.first_host),
                                        ip::to_dotted(info.last_host),
                                    );
                                    ctx_local.copy_text(all_text);
                                }
                                if ui.small_button("JSON").clicked() {
                                    let json = export::export_json_array(&[info.clone()]);
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        if let Some(path) = FileDialog::new()
                                            .add_filter("JSON Files", &["json"])
                                            .set_file_name(&format!("subnet_{}.json", i))
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
                            });
                        }
                    });
                }

                // 批量导出
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label("批量导出:");
                    if ui.button("JSON 数组").clicked() {
                        self.export_batch_json(ctx);
                    }
                    if ui.button("CSV").clicked() {
                        self.export_batch_csv(ctx);
                    }
                    if ui.button("Markdown").clicked() {
                        self.export_batch_markdown(ctx);
                    }
                });
            }
        } else {
            // ===== 单条模式 =====
            ui.horizontal(|ui| {
                ui.label("输入:");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(260.0)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("删除").clicked() {
                    self.input.clear();
                    self.recompute();
                }
                ui.label("(IP/CIDR 或 IP 掩码)");
                if resp.changed() {
                    self.recompute();
                }
            });

            if let Some(err) = &self.error {
                ui.colored_label(theme::ERROR, err);
            }

            if let Some(info) = &self.result {
                let info = *info;
                egui::Grid::new("basic_grid")
                    .num_columns(3)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        self.row(ui, ctx, "网络地址", &ip::to_dotted(info.network));
                        ui.end_row();

                        self.row(ui, ctx, "广播地址", &ip::to_dotted(info.broadcast));
                        ui.end_row();

                        self.row(ui, ctx, "子网掩码", &ip::to_dotted(info.mask));
                        ui.end_row();

                        self.row(ui, ctx, "反掩码", &ip::to_dotted(info.wildcard));
                        ui.end_row();

                        self.row(ui, ctx, "CIDR 表示", &subnet::cidr_representation(&info));
                        ui.end_row();

                        self.row(ui, ctx, "IP 类别", info.ip_class.description());
                        ui.end_row();

                        self.row(
                            ui,
                            ctx,
                            "是否私有",
                            if info.is_private { "是" } else { "否" },
                        );
                        ui.end_row();

                        self.row(ui, ctx, "IP 总数", &info.total_ips.to_string());
                        ui.end_row();

                        self.row(ui, ctx, "可用主机数", &info.usable_ips.to_string());
                        ui.end_row();

                        self.row(ui, ctx, "首个可用 IP", &ip::to_dotted(info.first_host));
                        ui.end_row();

                        self.row(ui, ctx, "末个可用 IP", &ip::to_dotted(info.last_host));
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label("掩码二进制:");
                let bin = mask_binary_grouped(&info);
                ui.horizontal(|ui| {
                    let mut bit_count: usize = 0;
                    for ch in bin.chars() {
                        let color = if ch == '.' {
                            theme::HOST_BIT
                        } else if bit_count < info.cidr as usize {
                            theme::NETWORK_BIT
                        } else {
                            theme::HOST_BIT
                        };
                        if ch != '.' {
                            bit_count += 1;
                        }
                        ui.label(
                            egui::RichText::new(ch.to_string())
                                .color(color)
                                .monospace()
                                .size(15.0)
                                .strong(),
                        );
                    }
                });

                // 导出按钮
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label("导出:");
                    if ui.button("复制全部").clicked() {
                        let all_text = format!(
                            "网络地址: {}\n广播地址: {}\n子网掩码: {}\n反掩码: {}\nCIDR 表示: {}\nIP 类别: {}\n是否私有: {}\nIP 总数: {}\n可用主机数: {}\n首个可用 IP: {}\n末个可用 IP: {}",
                            ip::to_dotted(info.network),
                            ip::to_dotted(info.broadcast),
                            ip::to_dotted(info.mask),
                            ip::to_dotted(info.wildcard),
                            subnet::cidr_representation(&info),
                            info.ip_class.description(),
                            if info.is_private { "是" } else { "否" },
                            info.total_ips,
                            info.usable_ips,
                            ip::to_dotted(info.first_host),
                            ip::to_dotted(info.last_host),
                        );
                        ctx.copy_text(all_text);
                        self.clipboard_msg = Some("已复制全部信息".to_string());
                        self.clipboard_msg_time = ctx.input(|i| i.time);
                    }
                    ui.separator();
                    if ui.button("JSON").clicked() {
                        let json = export::export_json(&info);
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Subnet Info", &["json"])
                                .set_file_name("subnet_info.json")
                                .save_file()
                            {
                                use std::fs;
                                use std::io::Write;
                                if let Ok(mut file) = fs::File::create(&path) {
                                    let _ = file.write_all(json.as_bytes());
                                }
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            ctx.copy_text(json.clone());
                            self.clipboard_msg = Some("已复制 JSON".to_string());
                            self.clipboard_msg_time = ctx.input(|i| i.time);
                        }
                    }
                    if ui.button("CSV").clicked() {
                        let csv = export::export_csv_line(&info);
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = FileDialog::new()
                                .add_filter("CSV Files", &["csv"])
                                .set_file_name("subnet_info.csv")
                                .save_file()
                            {
                                use std::fs;
                                use std::io::Write;
                                if let Ok(mut file) = fs::File::create(&path) {
                                    let _ = file.write_all(csv.as_bytes());
                                }
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            ctx.copy_text(csv.clone());
                            self.clipboard_msg = Some("已复制 CSV".to_string());
                            self.clipboard_msg_time = ctx.input(|i| i.time);
                        }
                    }
                    if ui.button("Markdown").clicked() {
                        let md = export::export_markdown_row(&info);
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Markdown Files", &["md"])
                                .set_file_name("subnet_info.md")
                                .save_file()
                            {
                                use std::fs;
                                use std::io::Write;
                                if let Ok(mut file) = fs::File::create(&path) {
                                    let _ = file.write_all(md.as_bytes());
                                }
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            ctx.copy_text(md.clone());
                            self.clipboard_msg = Some("已复制 Markdown".to_string());
                            self.clipboard_msg_time = ctx.input(|i| i.time);
                        }
                    }
                });
            }

            // 剪贴板消息提示
            if let Some(msg) = &self.clipboard_msg {
                let now = ctx.input(|i| i.time);
                if now - self.clipboard_msg_time < 1.5 {
                    ui.colored_label(theme::SUCCESS, msg);
                }
            }

            if let Some((text, t)) = &self.copied {
                let now = ctx.input(|i| i.time);
                if now - t < 1.5 {
                    ui.colored_label(theme::SUCCESS, format!("已复制: {}", text));
                }
            }
        }
    }

    fn row(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, label: &str, value: &str) {
        ui.label(label);
        ui.label(egui::RichText::new(value).monospace());
        if ui.button("复制").clicked() {
            ctx.copy_text(value.to_string());
            self.copied = Some((value.to_string(), ctx.input(|i| i.time)));
        }
    }

    fn recompute(&mut self) {
        if self.input.trim().is_empty() {
            self.result = None;
            self.error = None;
            return;
        }
        match subnet::analyze(&self.input) {
            Ok(info) => {
                self.result = Some(info);
                self.error = None;
                // 添加到历史记录
                self.history.add(&self.input, self.result.as_ref().unwrap().clone());
            }
            Err(e) => {
                self.result = None;
                self.error = Some(e.to_string());
            }
        }
    }

    fn batch_compute(&mut self) {
        self.results.clear();
        for line in self.batch_input.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match subnet::analyze(line) {
                Ok(info) => {
                    // Add to history
                    self.history.add(line, info.clone());
                    
                    self.results.push(ResultItem {
                        input_text: line.to_string(),
                        info,
                        error: None,
                    });
                }
                Err(e) => {
                    self.results.push(ResultItem {
                        input_text: line.to_string(),
                        info: self.empty_info(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    fn empty_info(&self) -> SubnetInfo {
        // Placeholder for error cases
        SubnetInfo {
            network: 0,
            broadcast: 0,
            mask: 0,
            wildcard: 0,
            cidr: 0,
            total_ips: 0,
            usable_ips: 0,
            first_host: 0,
            last_host: 0,
            ip_class: crate::core::ip::IpClass::ClassA,
            is_private: false,
        }
    }

    fn export_batch_json(&mut self, _ctx: &egui::Context) {
        let infos: Vec<SubnetInfo> = self.results.iter().filter(|r| r.error.is_none()).map(|r| r.info.clone()).collect();
        let json = export::export_json_array(&infos);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON Files", &["json"])
                .set_file_name("batch_results.json")
                .save_file()
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::File::create(&path) {
                    let _ = file.write_all(json.as_bytes());
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            ctx.copy_text(json);
        }
    }

    fn export_batch_csv(&mut self, _ctx: &egui::Context) {
        let infos: Vec<SubnetInfo> = self.results.iter().filter(|r| r.error.is_none()).map(|r| r.info.clone()).collect();
        let csv = export::export_csv_with_header(&infos);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .set_file_name("batch_results.csv")
                .save_file()
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::File::create(&path) {
                    let _ = file.write_all(csv.as_bytes());
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            ctx.copy_text(csv);
        }
    }

    fn export_batch_markdown(&mut self, _ctx: &egui::Context) {
        let infos: Vec<SubnetInfo> = self.results.iter().filter(|r| r.error.is_none()).map(|r| r.info.clone()).collect();
        let md = export::export_markdown_table(&infos);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Markdown Files", &["md"])
                .set_file_name("batch_results.md")
                .save_file()
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::File::create(&path) {
                    let _ = file.write_all(md.as_bytes());
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            ctx.copy_text(md);
        }
    }
}
