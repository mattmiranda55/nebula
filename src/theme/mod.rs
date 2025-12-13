// Nebula Theme - Space nebula inspired colors with purples, magentas, and cosmic hues
use eframe::egui::{self, Color32, Visuals};

// Background colors - deep space blacks with subtle purple tint
pub const BACKGROUND_DARKEST: Color32 = Color32::from_rgb(13, 10, 20);   // #0d0a14 - void black
pub const BACKGROUND_DARK: Color32 = Color32::from_rgb(20, 15, 31);      // #140f1f - deep space
pub const BACKGROUND_BASE: Color32 = Color32::from_rgb(26, 20, 38);      // #1a1426 - nebula dark
pub const BACKGROUND_LIGHT: Color32 = Color32::from_rgb(36, 28, 51);     // #241c33 - nebula mid
pub const BACKGROUND_LIGHTER: Color32 = Color32::from_rgb(46, 36, 64);   // #2e2440 - nebula light

// Primary accent - vibrant purple (main nebula color)
pub const PRIMARY: Color32 = Color32::from_rgb(153, 82, 230);            // #9952e6 - nebula purple
pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(186, 125, 242);     // #ba7df2 - light purple
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(115, 56, 179);       // #7338b3 - deep purple

// Secondary - cosmic pink/magenta
pub const SECONDARY: Color32 = Color32::from_rgb(232, 92, 163);          // #e85ca3 - cosmic pink

// Tertiary - cyan/teal for contrast (like star colors)
pub const TERTIARY: Color32 = Color32::from_rgb(77, 199, 230);           // #4dc7e6 - stellar cyan

// Text colors - starlight whites
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 235, 247);      // #f0ebf7 - starlight
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(179, 166, 199);    // #b3a6c7 - dim starlight
pub const TEXT_MUTED: Color32 = Color32::from_rgb(128, 115, 148);        // #807394 - distant stars

// Status colors
pub const SUCCESS: Color32 = Color32::from_rgb(102, 217, 153);           // #66d999 - aurora green
pub const WARNING: Color32 = Color32::from_rgb(242, 191, 89);            // #f2bf59 - solar flare
pub const DANGER: Color32 = Color32::from_rgb(242, 89, 115);             // #f25973 - red giant
pub const INFO: Color32 = Color32::from_rgb(102, 179, 242);              // #66b3f2 - blue star

// Borders - subtle purple-tinted
pub const BORDER: Color32 = Color32::from_rgb(64, 51, 89);               // #403359 - nebula edge

/// Creates the custom Nebula dark visuals for egui
pub fn dark_visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    
    visuals.panel_fill = BACKGROUND_BASE;
    visuals.window_fill = BACKGROUND_DARK;
    visuals.extreme_bg_color = BACKGROUND_DARKEST;
    visuals.faint_bg_color = BACKGROUND_LIGHT;
    
    visuals.widgets.noninteractive.bg_fill = BACKGROUND_LIGHT;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    
    visuals.widgets.inactive.bg_fill = BACKGROUND_LIGHT;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    
    visuals.widgets.hovered.bg_fill = BACKGROUND_LIGHTER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, PRIMARY);
    
    visuals.widgets.active.bg_fill = PRIMARY_DARK;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, PRIMARY);
    
    visuals.selection.bg_fill = PRIMARY_DARK;
    visuals.selection.stroke = egui::Stroke::new(1.0, PRIMARY);
    
    visuals.hyperlink_color = PRIMARY_LIGHT;
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = DANGER;
    
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    
    visuals.striped = true;
    
    visuals
}

