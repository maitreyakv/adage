use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, Ident, ItemFn, Pat, PatIdent, PatType, Type};

#[proc_macro_attribute]
pub fn task(
    _attr: proc_macro::TokenStream,
    task_ts: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let task_fn = TaskFn::parse(task_ts);
    let receiver_impl = task_fn.impl_receiver();
    let task_impl = task_fn.impl_task();

    quote! {
        #receiver_impl
        #task_impl
    }
    .into()
}

struct TaskFn(ItemFn);
impl TaskFn {
    fn parse(stream: proc_macro::TokenStream) -> Self {
        Self(syn::parse(stream).expect("adage::task should only decorate async functions!"))
    }

    fn name(&self) -> &Ident {
        &self.0.sig.ident
    }

    fn inputs(&self) -> impl Iterator<Item = TaskInput> {
        self.0
            .sig
            .inputs
            .iter()
            .cloned()
            .map(|fn_arg| match fn_arg {
                FnArg::Receiver(_) => panic!("adage::task cannot be used on struct methods!"),
                FnArg::Typed(pat_type) => TaskInput::parse(pat_type),
            })
    }

    fn receiver_name(&self) -> Ident {
        format_ident!(
            "{}Receiver",
            proc_macro2::Ident::new(
                &heck::AsUpperCamelCase(self.name().to_string()).to_string(),
                self.name().span()
            )
        )
    }

    fn impl_receiver(&self) -> TokenStream {
        let receiver_name = self.receiver_name();
        let input_name = format_ident!(
            "{}Input",
            proc_macro2::Ident::new(
                &heck::AsUpperCamelCase(self.name().to_string()).to_string(),
                self.name().span()
            )
        );

        let receiver_fields: Vec<TokenStream> = self
            .inputs()
            .map(|input| {
                let TaskInput { ident, ty } = input;
                quote! { #ident: ::tokio::sync::broadcast::Receiver<#ty> }
            })
            .collect();
        let input_fields: Vec<TokenStream> = self
            .inputs()
            .map(|input| {
                let TaskInput { ident, ty } = input;
                quote! { #ident: #ty }
            })
            .collect();
        let recv_stmts: Vec<TokenStream> = self
            .inputs()
            .map(|input| {
                let TaskInput { ident, ty: _ } = input;
                quote! { #ident: self.#ident.recv().await.unwrap() }
            })
            .collect();

        quote! {
            struct #receiver_name {
                #( #receiver_fields),*
            }
            struct #input_name {
                #( #input_fields ),*
            }
            impl ::adage::Receivable for #receiver_name {
                type Data = #input_name;
                async fn receive(mut self) -> Self::Data {
                    Self::Data {
                        #( #recv_stmts ),*

                    }
                }
            }
        }
    }

    fn impl_task(&self) -> TokenStream {
        let name = self.name();
        let inputs: Vec<TokenStream> = self
            .inputs()
            .map(|input| {
                let TaskInput { ident, ty } = input;
                quote! { #ident: &::adage::QueuedTask<#ty> }
            })
            .collect();

        let receiver_name = self.receiver_name();
        let link_stmts: Vec<TokenStream> = self
            .inputs()
            .map(|input| {
                let TaskInput { ident, ty: _ } = input;
                quote! { #ident: #ident.link() }
            })
            .collect();

        quote! {
            fn #name(#( #inputs),* ) {
                let receiver = #receiver_name {
                    #( #link_stmts ),*
                };
                ::adage::QueuedTask::new(receiver)
            }
        }
    }
}

struct TaskInput {
    ident: PatIdent,
    ty: Type,
}
impl TaskInput {
    fn parse(pat_type: PatType) -> Self {
        match *pat_type.pat {
            Pat::Ident(arg_ident) => {
                let arg_type = pat_type.ty;
                Self {
                    ident: arg_ident,
                    ty: *arg_type,
                }
            }
            _ => {
                panic!("adage::task only supports simple `name: Type` args!")
            }
        }
    }
}
