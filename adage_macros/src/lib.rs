use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Ident, ItemFn, ItemStruct, parse::Parser,
    punctuated::Punctuated, token::Comma,
};

#[proc_macro_attribute]
pub fn requires_context(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
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

    let item_struct: ItemStruct = syn::parse(item.clone()).unwrap();
    let item: TokenStream = item.into();
    let trait_ident = format_ident!("_{}Context", item_struct.ident);
    quote! {
        #item
        trait #trait_ident: #(#context_trait_bounds)+* {}
        impl<T> #trait_ident for T where T: #(#context_trait_bounds)+* {}
    }
    .into()
}

#[proc_macro_attribute]
pub fn provides(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = {
        let item = item.clone();
        syn::parse_macro_input!(item)
    };
    let alias_name = format_ident!("_{}Resource", item_struct.ident);
    let attr = TokenStream::from(attr);
    let item = TokenStream::from(item);
    quote! {
        #item
        type #alias_name = #attr;
    }
    .into()
}

#[proc_macro_attribute]
pub fn for_key(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = {
        let item = item.clone();
        syn::parse_macro_input!(item)
    };
    let alias_name = format_ident!("_{}Key", item_struct.ident);
    let attr = TokenStream::from(attr);
    let item = TokenStream::from(item);
    quote! {
        #item
        type #alias_name = #attr;
    }
    .into()
}

#[proc_macro_attribute]
pub fn provides_for(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let func: ItemFn = syn::parse_macro_input!(item);

    let layer_ident: Ident = syn::parse_macro_input!(attr);
    let context_trait_ident = format_ident!("_{}Context", layer_ident);
    let resource_type_ident = format_ident!("_{}Resource", layer_ident);
    let key_type_ident = format_ident!("_{}Key", layer_ident);
    let func_body = func.block.stmts;

    quote! {
        impl<C> ::adage::Layer<C> for #layer_ident
        where
            C: #context_trait_ident,
        {
            type Resource = #resource_type_ident;
            type Key = #key_type_ident;

            fn provide(&self, key: &Self::Key, ctx: C) -> Self::Resource {
                #(#func_body);*
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn ctx(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = TokenStream::from(match item.is_empty() {
        true => "()".parse().unwrap(),
        false => item,
    });
    quote! { ctx.get(#item) }.into()
}

#[proc_macro]
pub fn key(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! { key }.into()
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
        if parts.next().is_some() {
            panic!("Context requirement has more than two elements!")
        }

        Self { resource, key }
    }
}
