#[cfg(test)]
#[path = "skin_selection_test.rs"]
mod skin_selection_test;

use super::{ViewCommand, COLORTERM_ENV_VAR, TERM_ENV_VAR};
use crate::cli::skin::Skin;
use ratatui::prelude::Color;

impl ViewCommand {
    pub(super) fn select_skin(&self) -> Skin {
        let low_color = Skin {
            title_fg_color: Color::White,
            title_bg_color: Color::Blue,
            version_fg_color: Color::Gray,
            table_header_bg_color: Color::DarkGray,
            table_header_fg_color: Color::White,
            value_fg_color: None,
            value_style_reversed: true,
            delete_warning_text_fg_color: Color::LightRed,
            key_help_danger_bg_color: Color::LightRed,
            key_help_key_fg_color: Color::Gray,
            ..Default::default()
        };

        if let Some(color_count) = self.get_color_count() {
            match color_count {
                ..=256 => low_color,
                _ => Skin::default(),
            }
        } else {
            low_color
        }
    }

    pub(crate) fn get_color_count(&self) -> Option<u32> {
        if let Some(colorterm) = self
            .env_service
            .var(COLORTERM_ENV_VAR)
            .ok()
            .filter(|colorterm| !colorterm.is_empty())
            .or_else(|| {
                self.env_service
                    .var(TERM_ENV_VAR)
                    .ok()
                    .filter(|term| !term.is_empty())
            })
        {
            match colorterm.to_lowercase().as_str() {
                "truecolor" | "24bit" | "24-bit" => Some(16_777_216), // 24-bit color
                "kitty" | "kitty-256color" => Some(256),              // 256 colors
                "konsole" => Some(256),                               // 256 colors
                "rxvt-unicode-256color" => Some(256),                 // 256 colors
                "screen-256color" => Some(256),                       // 256 colors
                "tmux-256color" => Some(256),                         // 256 colors
                "xterm-256color" | "xterm256" => Some(256),           // 256 colors
                "ansi" => Some(16),                                   // 16 colors
                "screen" => Some(16),                                 // 16 colors
                "tmux" => Some(16),                                   // 16 colors
                "xterm" => Some(16),                                  // 16 colors
                "rxvt-unicode" => Some(8),                            // 8 colors (customizable)
                "dumb" => None,                                       // No color support
                "monochrome" => None,                                 // No color support
                _ => None,                                            // Unknown or custom value
            }
        } else {
            None
        }
    }
}
