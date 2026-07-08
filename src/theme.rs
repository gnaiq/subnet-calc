use eframe::egui::{Color32, FontData, FontDefinitions};

pub const BG: Color32 = Color32::from_rgb(30, 30, 36);
pub const ACCENT: Color32 = Color32::from_rgb(86, 156, 214);
pub const NETWORK_BIT: Color32 = Color32::from_rgb(86, 220, 156);
pub const HOST_BIT: Color32 = Color32::from_rgb(160, 160, 160);
pub const ERROR: Color32 = Color32::from_rgb(240, 100, 100);
pub const SUCCESS: Color32 = Color32::from_rgb(100, 200, 120);

pub fn setup_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = FontDefinitions::default();
    let bytes: &[u8] = include_bytes!("../assets/cjk_font.otf");
    fonts
        .font_data
        .insert("cjk".to_owned(), FontData::from_owned(bytes.to_vec()));
    fonts
        .families
        .entry(eframe::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "cjk".to_owned());
    fonts
        .families
        .entry(eframe::egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "cjk".to_owned());
    ctx.set_fonts(fonts);
}

pub fn setup_style(ctx: &eframe::egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = false;
    ctx.set_style(style);
}
