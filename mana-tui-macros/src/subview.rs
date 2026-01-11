use convert_case::Casing;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned};

pub struct SubviewFn {
    func: syn::ItemFn,
}

impl syn::parse::Parse for SubviewFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            func: input.parse()?,
        })
    }
}

fn mana_tui_elemental() -> proc_macro2::TokenStream {
    let found_crate =
        crate_name("mana-tui-elemental").expect("mana-tui-elemental is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    }
}

impl quote::ToTokens for SubviewFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SubviewFn { func } = self;
        let generics = &func.sig.generics;
        let func_name = &func.sig.ident;
        let name = {
            let func_name = func_name.to_string();
            let name = func_name.to_case(convert_case::Case::Pascal);
            let name = format_ident!("{name}");
            name
        };
        let mana_crate = mana_tui_elemental();
        let builder_module = format_ident!("{func_name}");
        let builder_generics = BuilderGenerics::new(generics);
        let span = func_name.span();

        let tok = quote_spanned! {
            span =>

            #[bon::builder(builder_type = #name)]
            #[builder(derive(Clone))]
            #[builder(finish_fn = into_view)]
            #func

            impl #generics Default for #name #generics {
                fn default() -> Self {
                    #func_name()
                }
            }

            impl #builder_generics From<#name #builder_generics> for #mana_crate::ui::View
            where
                S: #builder_module::IsComplete
            {
                fn from(value: #name #builder_generics) -> Self {
                    value.into_view()
                }
            }
        };
        tokens.extend(tok);
    }
}

struct BuilderGenerics(syn::Generics);

impl BuilderGenerics {
    fn new(initial: &syn::Generics) -> Self {
        let mut generics = initial.clone();
        generics
            .params
            .push_value(syn::GenericParam::Type(syn::TypeParam {
                attrs: Vec::default(),
                ident: format_ident!("S"),
                colon_token: None,
                bounds: syn::punctuated::Punctuated::new(),
                eq_token: None,
                default: None,
            }));
        Self(generics)
    }
}

impl quote::ToTokens for BuilderGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}
