use ratatui::style::Color;

#[derive(Clone, Copy)]
pub(crate) struct Skin {
    pub(crate) title_fg_color: Color,
    pub(crate) title_bg_color: Color,
    pub(crate) version_fg_color: Color,
    pub(crate) table_header_bg_color: Color,
    pub(crate) table_header_fg_color: Color,
    pub(crate) value_fg_color: Color,
    pub(crate) key_help_danger_bg_color: Color,
    pub(crate) key_help_key_fg_color: Color,
    pub(crate) item_type_directory_symbol: char,
    pub(crate) item_type_file_symbol: char,
    pub(crate) item_type_symbolic_link_symbol: char,
    pub(crate) item_type_unknown_symbol: char,
}

impl Default for Skin {
    fn default() -> Self {
        // The default dark mode skin
        Self {
            title_fg_color: Color::White,
            title_bg_color: Color::Rgb(64, 64, 64),
            version_fg_color: Color::Rgb(192, 192, 192),
            table_header_bg_color: Color::Rgb(64, 64, 176),
            table_header_fg_color: Color::White,
            value_fg_color: Color::Rgb(88, 144, 255),
            key_help_danger_bg_color: Color::Rgb(192, 64, 64),
            key_help_key_fg_color: Color::Rgb(192, 192, 192),
            item_type_directory_symbol: '📁',
            item_type_file_symbol: '📄',
            item_type_symbolic_link_symbol: '🔗',
            item_type_unknown_symbol: '❓',
        }
    }
}
