use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Expr, GenericArgument, ItemStruct, parse::Parser,
    punctuated::Punctuated, token::Comma,
};

#[proc_macro_attribute]
pub fn requires_context(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = {
        let item = item.clone();
        syn::parse_macro_input!(item)
    };

    let parser = Punctuated::<AngleBracketedGenericArguments, Comma>::parse_terminated;
    let context_requirements: Vec<_> = match parser.parse(attr) {
        Ok(p) => p,
        Err(e) => return e.into_compile_error().into(),
    }
    .into_iter()
    .map(ContextRequirement::from)
    .collect();

    let context_trait_bounds: Vec<_> = context_requirements
        .into_iter()
        .map(ContextRequirement::into_context_trait)
        .collect();

    let item: TokenStream = item.into();
    let trait_ident = format_ident!("_{}Context", item_struct.ident);
    quote! {
        #item
        trait #trait_ident: #(#context_trait_bounds)+* {}
        impl<T> #trait_ident for T where T: #(#context_trait_bounds)+* {}
    }
    .into()
}

#[derive(Debug)]
struct ContextRequirement {
    resource: TokenStream,
    key: Option<TokenStream>,
}

impl ContextRequirement {
    fn into_context_trait(self) -> TokenStream {
        let ContextRequirement { resource, key } = self;
        match key {
            Some(key) => quote! {
                for<'a> ::adage::Context<#resource, &'a #key>
            },
            None => quote! {
                ::adage::Context<#resource, ()>
            },
        }
    }
}

/// Allows parsing of ContextRequirement from expressions like `<SomeResource>` or
/// `<SomeResource, SomeKey>`
impl From<AngleBracketedGenericArguments> for ContextRequirement {
    fn from(value: AngleBracketedGenericArguments) -> Self {
        let mut parts = value.args.into_iter();
        let resource = parts
            .next()
            .expect("Error parsing context requirements!")
            .into_token_stream();
        let key = parts
            .next()
            .map_or(None, |part| Some(part.into_token_stream()));

        Self { resource, key }
    }
}
