use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, ItemFn, Pat, ReturnType};

#[proc_macro_attribute]
pub fn task(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_fn: ItemFn =
        syn::parse(item).expect("adage::task should only be used on async functions!");
    let task_name = item_fn.sig.ident;
    let body = item_fn.block;
    let return_type = match item_fn.sig.output {
        ReturnType::Default => quote! {()}.into_token_stream(),
        ReturnType::Type(_, type_) => type_.into_token_stream(),
    };

    let task_inputs: Vec<TokenStream> = item_fn
        .sig
        .inputs
        .clone()
        .into_iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Receiver(_) => panic!("adage::task cannot be used on struct methods!"),
            FnArg::Typed(pat_type) => {
                let name = match *pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.into_token_stream(),
                    _ => {
                        panic!(
                            "adage::task can only decorate functions with simple argument patterns!"
                        )
                    }
                };
                let type_ = pat_type.ty.into_token_stream();
                (name, type_)
            }
        })
        .map(|(name, type_)| quote! {#name: &::adage::QueuedTask<#type_>}.into())
        .collect();

    let link_stmts: Vec<TokenStream> = item_fn
        .sig
        .inputs
        .clone()
        .into_iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Receiver(_) => panic!("adage::task cannot be used on struct methods!"),
            FnArg::Typed(pat_type) => {
                let name = match *pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.into_token_stream(),
                    _ => {
                        panic!(
                            "adage::task can only decorate functions with simple argument patterns!"
                        )
                    }
                };
                let type_ = pat_type.ty.into_token_stream();
                (name, type_)
            }
        })
        .map(|(name, _)| quote! {let mut #name = #name.link();})
        .collect();

    let recv_stmts: Vec<TokenStream> = item_fn
        .sig
        .inputs
        .into_iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Receiver(_) => panic!("adage::task cannot be used on struct methods!"),
            FnArg::Typed(pat_type) => {
                let name = match *pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.into_token_stream(),
                    _ => {
                        panic!(
                            "adage::task can only decorate functions with simple argument patterns!"
                        )
                    }
                };
                let type_ = pat_type.ty.into_token_stream();
                (name, type_)
            }
        })
        .map(|(name, _)| quote! {let #name = #name.recv().await.unwrap();})
        .collect();

    quote! {
        fn #task_name(
            #(#task_inputs),*
        ) -> ::adage::QueuedTask<#return_type> {
            let (output_sender, dummy_receiver) = ::tokio::sync::broadcast::channel(1);
            let start_notifier = ::std::sync::Arc::new(::tokio::sync::Notify::new());
            let handle = {
                let start_notifier = start_notifier.clone();
                let output_sender = output_sender.clone();
                #(#link_stmts)*
                ::tokio::spawn(async move {
                    start_notifier.notified().await;
                    #(#recv_stmts)*
                    #[allow(clippy::let_unit_value)]
                    let output = #body;
                    output_sender.send(output).unwrap();
                    drop(dummy_receiver);
                })
            };
            ::adage::QueuedTask::new(handle, output_sender, start_notifier)
        }
    }
    .into()
}
