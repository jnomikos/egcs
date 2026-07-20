use egui::{Color32, Ui};

/// Reusuable color palette
pub const RED: Color32 = Color32::from_rgb(0xf4, 0x47, 0x47);
pub const GREEN: Color32 = Color32::from_rgb(0x6a, 0x99, 0x55);
pub const BLUE: Color32 = Color32::from_rgb(0x56, 0x9c, 0xd6);
pub const TEAL: Color32 = Color32::from_rgb(0x4e, 0xc9, 0xb0);
pub const AMBER: Color32 = Color32::from_rgb(0xd7, 0xba, 0x7d);

const WIDGET_WIDTH: f32 = 150.0;

// ITU-R BT.601 luma coefficients for converting RGB to luma (Y) in YUV color space.
// https://en.wikipedia.org/wiki/Luma_(video)
const LUMA_R: f32 = 0.299;
const LUMA_G: f32 = 0.587;
const LUMA_B: f32 = 0.114;
// Threshold for text on color background (0-1)
const LUMA_THRESHOLD: f32 = 0.55;

pub fn stat_tile(ui: &mut Ui, label: &str, value: impl Into<String>, color: egui::Color32) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_min_width(WIDGET_WIDTH);
                ui.label(egui::RichText::new(label).size(12.0).weak());
                ui.label(
                    egui::RichText::new(value.into())
                        .size(34.0)
                        .strong()
                        .color(color),
                );
            });
        });
}

pub fn action_button(ui: &mut Ui, label: &str, color: egui::Color32) -> bool {
    let luminance =
        LUMA_R * color.r() as f32 + LUMA_G * color.g() as f32 + LUMA_B * color.b() as f32;
    let luma_threshold = LUMA_THRESHOLD * 255.0;
    let text_color = if luminance > luma_threshold {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };
    let text = egui::RichText::new(label)
        .size(18.0)
        .strong()
        .color(text_color);
    let button = egui::Button::new((egui::Atom::grow(), text, egui::Atom::grow()))
        .fill(color)
        .corner_radius(6.0)
        .min_size(egui::vec2(WIDGET_WIDTH, 40.0));
    ui.add(button).clicked()
}
