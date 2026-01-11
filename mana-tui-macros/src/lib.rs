use manyhow::manyhow;
use quote::quote;
use syn::parse_macro_input;

mod manasx;
mod subview;

use crate::manasx::ManaElement;
use crate::subview::SubviewFn;

/// # Example
///
///```
/// use mana_tui_macros::ui;
/// use mana_tui_elemental::prelude::*;
///
/// let root = ui! {
///    <block .title_top="sidebar" Width(Size::Fixed(10)) Padding::uniform(1)>
///        <block .title_top="2" />
///    </block>
/// };
///```
#[manyhow]
#[proc_macro]
pub fn ui(input: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    // let input = preprocess_tokens(input.into());
    // let input = input.into();
    let tree = syn::parse::<ManaElement>(input)?;
    let tokens = quote! { #tree };

    Ok(tokens.into())
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
