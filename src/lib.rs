//! [`mana-tui-elemental`]: mana_tui_elemental
//!
#![doc = include_str!("../readme.md")]

pub use mana_tui_elemental;

pub mod prelude {
    pub use mana_tui_elemental::prelude::*;
    pub use mana_tui_macros::*;
    pub extern crate bon;
}
