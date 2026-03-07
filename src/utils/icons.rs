use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
pub enum IconSet {
    #[default]
    NerdFont,
    Unicode,
    Emoji,
    Ascii,
}

pub struct Icons {
    pub folder: &'static str,
    pub chat: &'static str,
    pub ok: &'static str,
    pub warn: &'static str,
    pub error: &'static str,
    pub risk: &'static str,
    pub unknown: &'static str,
}

impl Icons {
    pub fn get(set: IconSet) -> Self {
        match set {
            IconSet::NerdFont => Self {
                folder: "󰉋",
                chat: "󰭹",
                ok: "󰄵",
                warn: "󰀦",
                error: "󰅚",
                risk: "󰳦",
                unknown: "󰇊",
            },
            IconSet::Unicode => Self {
                folder: "📁",
                chat: "●",
                ok: "✓",
                warn: "▲",
                error: "✖",
                risk: "⚠",
                unknown: "○",
            },
            IconSet::Emoji => Self {
                folder: "📂",
                chat: "💬",
                ok: "✅",
                warn: "⚠️",
                error: "❌",
                risk: "🛡️",
                unknown: "❓",
            },
            IconSet::Ascii => Self {
                folder: "[P]",
                chat: "[S]",
                ok: "OK",
                warn: "!!",
                error: "XX",
                risk: "!!",
                unknown: "??",
            },
        }
    }
}
