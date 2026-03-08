use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Theme {
    pub border: Color,
    pub title: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub folder: Color,
    pub user_msg: Color,
    pub gemini_msg: Color,
    pub key_hint: Color,
    pub key_desc: Color,
}

impl Theme {
    pub fn tokyonight() -> Self {
        Self {
            border: Color::Rgb(65, 72, 104),
            title: Color::Rgb(122, 162, 247),
            selection_bg: Color::Rgb(47, 51, 73),
            selection_fg: Color::Rgb(255, 158, 100),
            folder: Color::Rgb(187, 154, 247),
            user_msg: Color::Rgb(122, 162, 247),
            gemini_msg: Color::Rgb(158, 206, 106),
            key_hint: Color::Rgb(187, 154, 247),
            key_desc: Color::Rgb(86, 95, 137),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            border: Color::Rgb(102, 92, 84),
            title: Color::Rgb(250, 189, 47),
            selection_bg: Color::Rgb(60, 56, 54),
            selection_fg: Color::Rgb(251, 73, 52),
            folder: Color::Rgb(131, 165, 152),
            user_msg: Color::Rgb(131, 165, 152),
            gemini_msg: Color::Rgb(184, 187, 38),
            key_hint: Color::Rgb(211, 134, 155),
            key_desc: Color::Rgb(146, 131, 116),
        }
    }

    pub fn onedark() -> Self {
        Self {
            border: Color::Rgb(75, 82, 99),
            title: Color::Rgb(97, 175, 239),
            selection_bg: Color::Rgb(44, 50, 60),
            selection_fg: Color::Rgb(224, 108, 117),
            folder: Color::Rgb(198, 120, 221),
            user_msg: Color::Rgb(97, 175, 239),
            gemini_msg: Color::Rgb(152, 195, 121),
            key_hint: Color::Rgb(198, 120, 221),
            key_desc: Color::Rgb(92, 99, 112),
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            border: Color::Rgb(88, 91, 112),
            title: Color::Rgb(137, 180, 250),
            selection_bg: Color::Rgb(49, 50, 68),
            selection_fg: Color::Rgb(243, 139, 168),
            folder: Color::Rgb(203, 166, 247),
            user_msg: Color::Rgb(137, 180, 250),
            gemini_msg: Color::Rgb(166, 227, 161),
            key_hint: Color::Rgb(245, 194, 231),
            key_desc: Color::Rgb(108, 112, 134),
        }
    }

    pub fn default_dark() -> Self {
        Self {
            border: Color::Gray,
            title: Color::Cyan,
            selection_bg: Color::DarkGray,
            selection_fg: Color::Yellow,
            folder: Color::Cyan,
            user_msg: Color::Blue,
            gemini_msg: Color::Green,
            key_hint: Color::Magenta,
            key_desc: Color::DarkGray,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ThemeConfig {
    Preset(String),
    Custom(Theme),
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::Preset("Default".to_string())
    }
}

impl ThemeConfig {
    pub fn get_theme(&self) -> Theme {
        match self {
            ThemeConfig::Custom(t) => *t,
            ThemeConfig::Preset(name) => match name.to_lowercase().as_str() {
                "tokyonight" => Theme::tokyonight(),
                "gruvbox" => Theme::gruvbox(),
                "onedark" => Theme::onedark(),
                "catppuccin" => Theme::catppuccin(),
                _ => Theme::default_dark(),
            },
        }
    }
}
