#![windows_subsystem = "windows"]

mod core;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 680.0])
            .with_title("子网掩码计算工具"),
        ..Default::default()
    };
    eframe::run_native(
        "子网掩码计算工具",
        options,
        Box::new(|_cc| Box::new(ui::app::App::default())),
    )
}
