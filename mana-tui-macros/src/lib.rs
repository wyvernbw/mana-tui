use quote::quote;
use syn::parse_macro_input;

mod manasx;
mod subview;
mod utils;

use crate::manasx::ManaElement;
use crate::subview::SubviewFn;
use crate::utils::mana_tui_elemental;

/// # Example
///
///```
/// use mana_tui_macros::ui;
/// use mana_tui::prelude::*;
///
/// let root = ui! {
///    <Block .title_top="sidebar" Width(Size::Fixed(10)) Padding::uniform(1)>
///        <Block .title_top="2" />
///    </Block>
/// };
///```
#[proc_macro]
pub fn ui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = preprocess_tokens(input.into());
    // let input = input.into();
    let tree = parse_macro_input!(input as ManaElement);
    let tree = quote! { #tree };
    let mana_crate = mana_tui_elemental();
    let tokens = quote! {
        {
            use #mana_crate::ui::__ui_internal;
            #tree
        }
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn subview(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let subview = parse_macro_input!(item as SubviewFn);
    let tok = quote! { #subview };
    tok.into()
}
