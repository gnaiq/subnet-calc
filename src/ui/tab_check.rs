use eframe::egui;

use crate::core::aggregate;
use crate::core::subnet::{self, SubnetInfo};
use crate::theme;

pub struct CheckState {
    pub input_a: String,
    pub input_b: String,
    pub multi_input: String,
    pub multi_result: Option<Vec<SubnetInfo>>,
    pub multi_error: Option<String>,
    pub copied: Option<(String, f64)>,
}

impl Default for CheckState {
    fn default() -> Self {
        Self {
            input_a: "192.168.0.0/16".to_string(),
            input_b: "192.168.1.0/24".to_string(),
            multi_input: "192.168.0.0/24\n192.168.1.0/24\n192.168.2.0/24\n192.168.3.0/24"
                .to_string(),
            multi_result: None,
            multi_error: None,
            copied: None,
        }
    }
}

impl CheckState {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical()
            .id_source("check_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.show_inner(ui, ctx);
            });
    }

    fn show_inner(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label("两网段关系判断");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("网段 A:");
            ui.add(
                egui::TextEdit::singleline(&mut self.input_a)
                    .desired_width(220.0)
                    .font(egui::TextStyle::Monospace),
            );
            ui.label("网段 B:");
            ui.add(
                egui::TextEdit::singleline(&mut self.input_b)
                    .desired_width(220.0)
                    .font(egui::TextStyle::Monospace),
            );
        });

        ui.add_space(6.0);

        let a = subnet::analyze(&self.input_a);
        let b = subnet::analyze(&self.input_b);

        match (&a, &b) {
            (Ok(a), Ok(b)) => {
                let a_contains_b = aggregate::contains(a, b);
                let b_contains_a = aggregate::contains(b, a);
                let overlap = aggregate::overlaps(a, b);
                let agg = aggregate::can_aggregate(a, b);

                check_row(ui, "A 包含 B ?", a_contains_b);
                check_row(ui, "B 包含 A ?", b_contains_a);
                check_row(ui, "是否重叠 ?", overlap);

                if let Some(agg_info) = &agg {
                    ui.add_space(4.0);
                    let val = subnet::cidr_representation(agg_info);
                    ui.horizontal(|ui| {
                        ui.label("可聚合为:");
                        ui.label(
                            egui::RichText::new(&val)
                                .monospace()
                                .strong()
                                .color(theme::ACCENT),
                        );
                        if ui.button("复制").clicked() {
                            ctx.copy_text(val.clone());
                            self.copied = Some((val, ctx.input(|i| i.time)));
                        }
                    });
                } else {
                    let reason = if a.cidr != b.cidr {
                        "前缀长度不同".to_string()
                    } else if a.cidr == 0 {
                        "/0 不可再聚合".to_string()
                    } else if a.network == b.network {
                        "相同网段".to_string()
                    } else {
                        "不在同一超网".to_string()
                    };
                    check_row_text(ui, "能否聚合 ?", &format!("否（{}）", reason), false);
                }
            }
            (Err(e), _) => {
                ui.colored_label(theme::ERROR, format!("网段 A 错误: {}", e));
            }
            (_, Err(e)) => {
                ui.colored_label(theme::ERROR, format!("网段 B 错误: {}", e));
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label("多网段路由汇总");
        ui.add_space(4.0);

        ui.add(
            egui::TextEdit::multiline(&mut self.multi_input)
                .desired_width(f32::MAX)
                .desired_rows(5)
                .font(egui::TextStyle::Monospace),
        );

        if ui.button("汇总").clicked() {
            self.aggregate_multi();
        }

        if let Some(err) = &self.multi_error {
            ui.colored_label(theme::ERROR, err);
        }

        if let Some(results) = &self.multi_result {
            ui.add_space(6.0);
            ui.label(format!("汇总结果 ({}):", results.len()));
            egui::Grid::new("multi_result")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("#");
                    ui.label("聚合网段");
                    ui.end_row();
                    for (i, net) in results.iter().enumerate() {
                        ui.label((i + 1).to_string());
                        let val = subnet::cidr_representation(net);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&val).monospace());
                            if ui.button("复制").clicked() {
                                ctx.copy_text(val.clone());
                                self.copied = Some((val, ctx.input(|i| i.time)));
                            }
                        });
                        ui.end_row();
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

    fn aggregate_multi(&mut self) {
        let mut nets = Vec::new();
        let mut errors = Vec::new();
        for (i, line) in self.multi_input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match subnet::analyze(line) {
                Ok(n) => nets.push(n),
                Err(e) => errors.push(format!("第{}行: {}", i + 1, e)),
            }
        }
        if !errors.is_empty() {
            self.multi_result = None;
            self.multi_error = Some(errors.join("; "));
            return;
        }
        if nets.is_empty() {
            self.multi_result = None;
            self.multi_error = Some("无有效网段".to_string());
            return;
        }
        let result = aggregate::aggregate_many(nets);
        self.multi_result = Some(result);
        self.multi_error = None;
    }
}

fn check_row(ui: &mut egui::Ui, question: &str, yes: bool) {
    let (color, answer) = if yes {
        (theme::SUCCESS, "是")
    } else {
        (theme::ERROR, "否")
    };
    ui.horizontal(|ui| {
        ui.label(question);
        ui.label(egui::RichText::new(answer).color(color).strong());
    });
}

fn check_row_text(ui: &mut egui::Ui, question: &str, answer: &str, positive: bool) {
    let color = if positive {
        theme::SUCCESS
    } else {
        theme::ERROR
    };
    ui.horizontal(|ui| {
        ui.label(question);
        ui.label(egui::RichText::new(answer).color(color));
    });
}
