use eframe::egui;

use crate::core::ip;
use crate::core::subnet::{self, mask_binary_grouped, SubnetInfo};
use crate::theme;

pub struct BasicState {
    pub input: String,
    pub result: Option<SubnetInfo>,
    pub error: Option<String>,
    pub copied: Option<(String, f64)>,
}

impl Default for BasicState {
    fn default() -> Self {
        Self {
            input: "192.168.1.1/24".to_string(),
            result: None,
            error: None,
            copied: None,
        }
    }
}

impl BasicState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("输入:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .desired_width(300.0)
                    .font(egui::TextStyle::Monospace),
            );
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
        }

        if let Some((text, t)) = &self.copied {
            let now = ctx.input(|i| i.time);
            if now - t < 1.5 {
                ui.colored_label(theme::SUCCESS, format!("已复制: {}", text));
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
            }
            Err(e) => {
                self.result = None;
                self.error = Some(e.to_string());
            }
        }
    }
}
