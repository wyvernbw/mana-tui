//! [`mana-tui-elemental`]: mana_tui_elemental
//!
#![doc = include_str!("../readme.md")]

pub use mana_tui_beheaded;
pub use mana_tui_elemental;
pub use mana_tui_utils;

pub mod prelude {
    pub use mana_tui_elemental::prelude::*;

    #[cfg(feature = "macros")]
    pub use mana_tui_macros::*;
    #[cfg(feature = "macros")]
    pub extern crate bon;

    pub use ratatui::style::palette;
    pub use ratatui::style::*;
}

pub use mx_core as mx;

// TODO: lock behind crossterm feature
#[macro_export]
macro_rules! key {
    // Variant with no arguments, e.g. Enter, Esc
    ($code:ident, $kind:ident) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        }
    };
    // Variant with arguments, e.g. Char('x'), Char(_)
    ($code:ident ( $($arg:tt)* ), $kind:ident) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        }
    };
    // Variant with no arguments and optional modifiers
    ($code:ident, $kind:ident, $mods:expr ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: $mods,
            ..
        }
    };
    ($code:ident ( $($arg:tt)* ), $kind:ident, $mods:pat) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: $mods,
            ..
        }
    };
}
