use eframe::egui;

use crate::ui::{tab_basic, tab_check, tab_convert, tab_history, tab_vlsm};

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Basic,
    Vlsm,
    Convert,
    Check,
    History,
}

pub struct App {
    pub tab: Tab,
    pub basic: tab_basic::BasicState,
    pub vlsm: tab_vlsm::VlsmState,
    pub convert: tab_convert::ConvertState,
    pub check: tab_check::CheckState,
    pub history: tab_history::HistoryState,
    pub clipboard_msg: Option<String>,
    pub clipboard_msg_time: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::Basic,
            basic: Default::default(),
            vlsm: Default::default(),
            convert: Default::default(),
            check: Default::default(),
            history: Default::default(),
            clipboard_msg: None,
            clipboard_msg_time: 0.0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::theme::setup_style(ctx);
        crate::theme::setup_fonts(ctx);

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Basic, "基础计算");
                ui.selectable_value(&mut self.tab, Tab::Vlsm, "VLSM 划分");
                ui.selectable_value(&mut self.tab, Tab::Convert, "格式转换");
                ui.selectable_value(&mut self.tab, Tab::Check, "包含与聚合");
                ui.selectable_value(&mut self.tab, Tab::History, "历史记录");
            });
            ui.separator();
        });

        if let Some(msg) = &self.clipboard_msg {
            let now = ctx.input(|i| i.time);
            if now - self.clipboard_msg_time < 1.5 {
                egui::TopBottomPanel::bottom("clipboard_msg").show(ctx, |ui| {
                    ui.colored_label(crate::theme::SUCCESS, msg);
                });
            } else {
                self.clipboard_msg = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Basic => self.basic.show(ui, ctx),
            Tab::Vlsm => self.vlsm.show(ui, ctx),
            Tab::Convert => self.convert.show(ui, ctx),
            Tab::Check => self.check.show(ui, ctx),
            Tab::History => self.history.show(ui, ctx),
        });
    }
}
