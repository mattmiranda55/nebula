// Nebula Theme - Space nebula inspired colors with purples, magentas, and cosmic hues
use iced::Theme;

/// Space nebula inspired color palette
pub mod colors {
    use iced::Color;

    // Background colors - deep space blacks with subtle purple tint
    pub const BACKGROUND_DARKEST: Color = Color::from_rgb(0.05, 0.04, 0.08); // #0d0a14 - void black
    pub const BACKGROUND_DARK: Color = Color::from_rgb(0.08, 0.06, 0.12); // #140f1f - deep space
    pub const BACKGROUND_BASE: Color = Color::from_rgb(0.10, 0.08, 0.15); // #1a1426 - nebula dark
    pub const BACKGROUND_LIGHT: Color = Color::from_rgb(0.14, 0.11, 0.20); // #241c33 - nebula mid
    pub const BACKGROUND_LIGHTER: Color = Color::from_rgb(0.18, 0.14, 0.25); // #2e2440 - nebula light

    // Primary accent - vibrant purple (main nebula color)
    pub const PRIMARY: Color = Color::from_rgb(0.60, 0.32, 0.90); // #9952e6 - nebula purple
    pub const PRIMARY_LIGHT: Color = Color::from_rgb(0.73, 0.49, 0.95); // #ba7df2 - light purple
    pub const PRIMARY_DARK: Color = Color::from_rgb(0.45, 0.22, 0.70); // #7338b3 - deep purple

    // Secondary - cosmic pink/magenta
    pub const SECONDARY: Color = Color::from_rgb(0.91, 0.36, 0.64); // #e85ca3 - cosmic pink
    pub const SECONDARY_LIGHT: Color = Color::from_rgb(0.95, 0.55, 0.75); // #f28cbf - soft pink

    // Tertiary - cyan/teal for contrast (like star colors)
    pub const TERTIARY: Color = Color::from_rgb(0.30, 0.78, 0.90); // #4dc7e6 - stellar cyan
    pub const TERTIARY_LIGHT: Color = Color::from_rgb(0.50, 0.88, 0.95); // #80e0f2 - bright cyan

    // Text colors - starlight whites
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.94, 0.92, 0.97); // #f0ebf7 - starlight
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.70, 0.65, 0.78); // #b3a6c7 - dim starlight
    pub const TEXT_MUTED: Color = Color::from_rgb(0.50, 0.45, 0.58); // #807394 - distant stars

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(0.40, 0.85, 0.60); // #66d999 - aurora green
    pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.35); // #f2bf59 - solar flare
    pub const DANGER: Color = Color::from_rgb(0.95, 0.35, 0.45); // #f25973 - red giant
    pub const INFO: Color = Color::from_rgb(0.40, 0.70, 0.95); // #66b3f2 - blue star

    // Syntax highlighting colors - cosmic palette
    pub const SYNTAX_KEYWORD: Color = Color::from_rgb(0.73, 0.49, 0.95); // #ba7df2 - purple
    pub const SYNTAX_STRING: Color = Color::from_rgb(0.40, 0.85, 0.60); // #66d999 - green
    pub const SYNTAX_NUMBER: Color = Color::from_rgb(0.95, 0.75, 0.35); // #f2bf59 - gold
    pub const SYNTAX_COMMENT: Color = Color::from_rgb(0.50, 0.45, 0.58); // #807394 - muted
    pub const SYNTAX_FUNCTION: Color = Color::from_rgb(0.30, 0.78, 0.90); // #4dc7e6 - cyan
    pub const SYNTAX_TABLE: Color = Color::from_rgb(0.91, 0.36, 0.64); // #e85ca3 - pink

    // Borders - subtle purple-tinted
    pub const BORDER: Color = Color::from_rgb(0.25, 0.20, 0.35); // #403359 - nebula edge
    pub const BORDER_LIGHT: Color = Color::from_rgb(0.35, 0.28, 0.45); // #594773 - bright edge
}

/// IBM Plex Mono font
pub mod fonts {
    use iced::Font;

    pub const IBM_PLEX_MONO: Font = Font::with_name("IBM Plex Mono");
    pub const IBM_PLEX_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/IBM_Plex_Mono/IBMPlexMono-Regular.ttf");
    pub const IBM_PLEX_MONO_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/IBM_Plex_Mono/IBMPlexMono-Bold.ttf");
}

/// Creates the custom Nebula dark theme
pub fn nebula_theme() -> Theme {
    Theme::custom(
        "Nebula".to_string(),
        iced::theme::Palette {
            background: colors::BACKGROUND_BASE,
            text: colors::TEXT_PRIMARY,
            primary: colors::PRIMARY,
            success: colors::SUCCESS,
            warning: colors::WARNING,
            danger: colors::DANGER,
        },
    )
}

/// Widget style constants
pub mod styles {
    use super::colors;
    use iced::widget::container;
    use iced::{Background, Border, Color, Theme};

    // Sidebar panel style
    pub fn sidebar_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_DARK)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }

    // Main content area style
    pub fn content_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_BASE)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }

    // Toolbar style
    pub fn toolbar_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_DARKEST)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }

    // Panel with border style
    pub fn bordered_panel(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_LIGHT)),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: colors::BORDER,
            },
            ..Default::default()
        }
    }

    // Table row style (even)
    pub fn table_row_even(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_BASE)),
            ..Default::default()
        }
    }

    // Table row style (odd)
    pub fn table_row_odd(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_LIGHT)),
            ..Default::default()
        }
    }

    // Table header style
    pub fn table_header(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_DARKEST)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}
