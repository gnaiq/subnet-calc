use eframe::egui;

use crate::core::ip;
use crate::core::mask;
use crate::theme;

pub struct ConvertState {
    pub input: String,
    pub copied: Option<(String, f64)>,
}

impl Default for ConvertState {
    fn default() -> Self {
        Self {
            input: "192.168.1.1".to_string(),
            copied: None,
        }
    }
}

impl ConvertState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("输入:");
            ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .desired_width(300.0)
                    .font(egui::TextStyle::Monospace),
            );
            ui.label("(IP 或掩码)");
        });

        ui.add_space(8.0);

        let trimmed = self.input.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        if let Ok(val) = ip::parse_ipv4(&trimmed) {
            ui.label("IPv4 地址:");
            egui::Grid::new("convert_ip_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    self.row(ui, ctx, "点分十进制", &ip::to_dotted(val));
                    ui.end_row();
                    self.row(ui, ctx, "整数", &ip::to_integer(val));
                    ui.end_row();
                    self.row(ui, ctx, "二进制", &ip::to_binary(val));
                    ui.end_row();
                    self.row(ui, ctx, "十六进制", &ip::to_hex(val));
                    ui.end_row();
                });
        } else if let Ok(cidr) = mask::parse_mask(&trimmed) {
            let m = mask::cidr_to_mask(cidr);
            ui.label(format!("子网掩码 (/{})", cidr));
            egui::Grid::new("convert_mask_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    self.row(ui, ctx, "点分十进制", &mask::to_dotted(m));
                    ui.end_row();
                    self.row(ui, ctx, "CIDR", &cidr.to_string());
                    ui.end_row();
                    self.row(ui, ctx, "反掩码", &mask::to_dotted_wildcard(m));
                    ui.end_row();
                    self.row(ui, ctx, "二进制", &ip::to_binary(m));
                    ui.end_row();
                });
        } else {
            ui.colored_label(theme::ERROR, "无法解析为 IP 或掩码");
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
}
