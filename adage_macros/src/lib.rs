use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, Ident, ItemFn, Pat, PatIdent, PatType, ReturnType};

#[proc_macro_attribute]
pub fn task(
    _attr: proc_macro::TokenStream,
    task_ts: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let task_item_fn = TaskItemFn::parse(task_ts);
    let input_struct = impl_input_struct(&task_item_fn);
    let input_receiver_struct = impl_input_receiver_struct(&task_item_fn);
    let input_receiver_trait_impl = impl_input_receiver_trait_impl(&task_item_fn);
    let task_fn_struct_and_impl = impl_task_fn_struct_and_impl(&task_item_fn);
    let functional_constructor = impl_functional_constructor(&task_item_fn);
    quote! {
        #input_struct
        #input_receiver_struct
        #input_receiver_trait_impl
        #task_fn_struct_and_impl
        #functional_constructor
    }
    .into()
}

struct TaskItemFn(ItemFn);
impl Deref for TaskItemFn {
    type Target = ItemFn;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TaskItemFn {
    fn parse(stream: proc_macro::TokenStream) -> Self {
        Self(syn::parse(stream).expect("adage::task should only decorate async functions!"))
    }

    fn args(&self) -> impl Iterator<Item = TaskPatType> {
        self.sig.inputs.iter().cloned().map(|fn_arg| match fn_arg {
            FnArg::Receiver(_) => panic!("adage::task cannot be used on struct methods!"),
            FnArg::Typed(pat_type) => TaskPatType(pat_type),
        })
    }

    fn format_camel_ident(&self, suffix: &str) -> Ident {
        format_ident!(
            "{}{}",
            heck::AsUpperCamelCase(self.sig.ident.to_string()).to_string(),
            suffix
        )
    }
}

struct TaskPatType(PatType);
impl Deref for TaskPatType {
    type Target = PatType;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TaskPatType {
    fn name(&self) -> PatIdent {
        match *self.pat.to_owned() {
            Pat::Ident(pat_ident) => pat_ident,
            _ => panic!("adage::task only support simple task arguments (e.g. `name: Type`)!"),
        }
    }
}

fn impl_input_struct(task_item_fn: &TaskItemFn) -> TokenStream {
    let ident = task_item_fn.format_camel_ident("Input");
    let fields = task_item_fn.args().map(|tpt| PatType {
        attrs: Vec::new(),
        ..tpt.clone()
    });
    quote! { struct #ident { #( #fields ),* } }
}

fn impl_input_receiver_struct(task_item_fn: &TaskItemFn) -> TokenStream {
    let ident = task_item_fn.format_camel_ident("InputReceiver");
    let fields = task_item_fn.args().map(|tpt| {
        let name = tpt.name();
        let ty = tpt.ty.clone();
        quote! { #name: ::tokio::sync::broadcast::Receiver<#ty> }
    });
    quote! { struct #ident { #( #fields ),* } }
}

fn impl_input_receiver_trait_impl(task_item_fn: &TaskItemFn) -> TokenStream {
    let input_receiver_ident = task_item_fn.format_camel_ident("InputReceiver");
    let input_ident = task_item_fn.format_camel_ident("Input");
    let recv_stmts = task_item_fn.args().map(|tpt| {
        let name = tpt.name();
        quote! { #name: self.#name.recv().await.unwrap() }
    });
    quote! {
        impl ::adage::prelude::InputReceiver for #input_receiver_ident {
            type Data = #input_ident;
            type Error = ::std::convert::Infallible;
            async fn try_recv(mut self) -> Result<Self::Data, Self::Error> {
                Ok(Self::Data { #( #recv_stmts ),* })
            }
        }
    }
}

fn impl_task_fn_struct_and_impl(task_item_fn: &TaskItemFn) -> TokenStream {
    let task_fn_ident = task_item_fn.format_camel_ident("TaskFn");
    let input_ident = task_item_fn.format_camel_ident("Input");
    let input_fields = task_item_fn.args().map(|tpt| tpt.name());
    let output: TokenStream = match &task_item_fn.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => ty.to_owned().to_token_stream(),
    };
    let body = task_item_fn.block.stmts.to_owned();
    quote! {
        struct #task_fn_ident;
        impl ::adage::prelude::TaskFn for #task_fn_ident {
            type Input = #input_ident;
            type Output = #output;
            async fn run(input: Self::Input) -> Self::Output {
                let #input_ident { #( #input_fields ),* } = input;
                #( #body )*
            }
        }
    }
}

fn impl_functional_constructor(task_item_fn: &TaskItemFn) -> TokenStream {
    let constructor_ident = task_item_fn.sig.ident.to_owned();
    let constructor_args = task_item_fn.args().map(|tpt| {
        let name = tpt.name();
        let ty = tpt.ty.to_owned();
        quote!(#name: ::adage::prelude::Linker<#ty>)
    });
    let task_fn_ident = task_item_fn.format_camel_ident("TaskFn");
    let input_receiver_ident = task_item_fn.format_camel_ident("InputReceiver");
    let link_stmts = task_item_fn.args().map(|tpt| {
        let name = tpt.name();
        quote!(#name: #name.link())
    });
    quote! {
        fn #constructor_ident(
            #( #constructor_args ),*
        ) -> ::adage::prelude::PlannedTask<#task_fn_ident, #input_receiver_ident> {
            let input_receiver = #input_receiver_ident {
                #( #link_stmts ),*
            };
            ::adage::prelude::PlannedTask::new(input_receiver)
        }
    }
}
