use convert_case::Casing;
use quote::{format_ident, quote_spanned};
use syn::{PatType, spanned::Spanned};

use crate::utils::mana_tui_elemental;

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

impl quote::ToTokens for SubviewFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SubviewFn { func } = self;
        let generics = &func.sig.generics;
        let impl_trait_params = func
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat_type) => Some(pat_type),
            })
            .filter_map(|arg| match *arg.ty.clone() {
                syn::Type::ImplTrait(ty) => Some((arg, ty)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let func_name = &func.sig.ident;
        let name = {
            let func_name = func_name.to_string();
            let name = func_name.to_case(convert_case::Case::Pascal);
            let name = format_ident!("{name}");
            name
        };
        let mana_crate = mana_tui_elemental();
        let builder_module = format_ident!("{func_name}");

        let mut generics = generics.clone();
        for (idx, (ty, impl_trait)) in impl_trait_params.iter().enumerate() {
            let ident = format_ident!("T{idx}");
            generics
                .params
                .push(syn::GenericParam::Type(syn::TypeParam {
                    attrs: ty.attrs.clone(),
                    ident,
                    colon_token: Some(syn::token::Colon::default()),
                    bounds: impl_trait.bounds.clone(),
                    eq_token: None,
                    default: None,
                }));
        }
        let builder_generics = BuilderGenerics::new(&generics, func, &builder_module);
        let (impl_generics, ty_generics, where_clause) =
            builder_generics.full_generics.split_for_impl();
        let (base_impl, base_ty, base_wh) = builder_generics.implicit_lifetimes.split_for_impl();
        let where_clause_is_complete = where_clause.cloned().map(|mut wh| {
            let mut complete_bound = syn::punctuated::Punctuated::new();
            complete_bound.push(syn::TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: syn::parse2(quote::quote! {#builder_module::IsComplete}).unwrap(),
            }));
            wh.predicates
                .push(syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Verbatim(quote::quote! {S}),
                    colon_token: syn::token::Colon::default(),
                    bounds: complete_bound,
                }));
            wh
        });
        let span = func_name.span();

        let tok = quote_spanned! {
            span =>

            #[bon::builder(builder_type = #name)]
            #[builder(finish_fn = into_view)]
            #[builder(crate = ::mana_tui::prelude::bon)]
            #func

            impl #base_impl Default for #name #base_ty
            #base_wh
            {
                fn default() -> Self {
                    #func_name()
                }
            }

            impl #impl_generics From<#name #ty_generics> for #mana_crate::ui::View
            #where_clause_is_complete
            {
                fn from(value: #name #ty_generics) -> Self {
                    value.into_view()
                }
            }
        };
        tokens.extend(tok);
    }
}

#[derive(Debug, Clone)]
struct BuilderGenerics {
    implicit_lifetimes: syn::Generics,
    full_generics: syn::Generics,
}

impl BuilderGenerics {
    fn new(initial: &syn::Generics, func_def: &syn::ItemFn, builder_module: &syn::Ident) -> Self {
        let mut lifetimes = initial.clone();
        let mut implicit_lifetimes = 0;
        for arg in &func_def.sig.inputs {
            match arg {
                syn::FnArg::Receiver(_) => {}
                syn::FnArg::Typed(syn::PatType { ty, .. }) => {
                    if let syn::Type::Reference(reference) = ty.as_ref()
                        && reference.lifetime.is_none()
                    {
                        implicit_lifetimes += 1;
                        let ident = format!("'f{implicit_lifetimes}");
                        lifetimes.params.push(syn::GenericParam::Lifetime(
                            syn::LifetimeParam::new(syn::Lifetime::new(&ident, reference.span())),
                        ));
                    }
                }
            }
        }
        let mut generics = lifetimes.clone();
        generics
            .params
            .push(syn::GenericParam::Type(syn::TypeParam {
                attrs: Vec::default(),
                ident: format_ident!("S"),
                colon_token: None,
                bounds: syn::punctuated::Punctuated::new(),
                eq_token: None,
                default: None,
            }));

        let mut where_clause = generics.where_clause.unwrap_or_else(|| syn::WhereClause {
            where_token: syn::token::Where::default(),
            predicates: syn::punctuated::Punctuated::new(),
        });
        let mut bounds = syn::punctuated::Punctuated::new();
        bounds.push(syn::TypeParamBound::Verbatim(quote::quote! {
            #builder_module::State
        }));
        where_clause
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: syn::Type::Verbatim(quote::quote! {S}),
                colon_token: syn::token::Colon::default(),
                bounds,
            }));
        _ = where_clause.predicates.pop_punct();
        generics.where_clause = Some(where_clause);
        Self {
            full_generics: generics,
            implicit_lifetimes: lifetimes,
        }
    }
}

impl quote::ToTokens for BuilderGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.full_generics.to_tokens(tokens);
    }
}
