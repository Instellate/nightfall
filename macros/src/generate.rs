use proc_macro2::TokenStream;
use quote::quote;
use std::ops::Deref;
use syn::spanned::Spanned;
use syn::{ImplItem, ImplItemFn, ItemImpl};

pub(crate) fn generate_command_controller(
    impl_: ItemImpl,
    args: crate::CommandControllerConfig,
) -> Result<TokenStream, Box<dyn crate::Error>> {
    let fn_items: Vec<&ImplItemFn> = impl_
        .items
        .iter()
        .filter_map(|m| match m {
            ImplItem::Fn(i) => Some(i),
            _ => None,
        })
        .collect();

    let struct_name = if let syn::Type::Path(p) = impl_.self_ty.deref() {
        p
    } else {
        return Err(Box::new(syn::Error::new(impl_.self_ty.span(), "")));
    };

    let mut statements = quote! {};
    for fn_item in fn_items {
        let ident = &fn_item.sig.ident;
        let name = fn_item.sig.ident.to_string();

        let mut args: Vec<TokenStream> = vec![];
        let mut is_self = false;
        for arg in fn_item.sig.inputs.iter() {
            match arg {
                syn::FnArg::Typed(t) => t,
                syn::FnArg::Receiver(r) => {
                    if r.mutability.is_some() {
                        return Err(Box::new(syn::Error::new(
                            arg.span(),
                            "Self must be non mutable reference",
                        )));
                    } else if r.reference.is_none() {
                        return Err(Box::new(syn::Error::new(
                            arg.span(),
                            "Self must be reference",
                        )));
                    }

                    is_self = true;
                    continue;
                }
            };

            args.push(quote! {
                match data.options.iter().find(|o| o.name == "") {
                    Some(v) => match FromOption::from_option(v.value.clone()) {
                        Some(v2) => v2,
                        None => return Err(Error::OptionBindingFailed)
                    },
                    None => return Err(Error::OptionBindingFailed),
                }
            });
        }

        let call = if is_self {
            quote! { self.#ident(#(#args),*).await }
        } else {
            quote! { Self::#ident(#(#args),*).await }
        };

        statements = quote! {
            #statements
            if data.name == #name {
                match #call {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(::nightfall::Error::CommandError { error: e }),
                };
            }
        }
    }

    Ok(quote! {
        //#impl_

        #[::nightfall::export::async_trait::async_trait]
        impl ::nightfall::CommandController for #struct_name {
            async fn execute_command(
                &self,
                interaction: &::nightfall::export::twilight_model::gateway::payload::incoming::InteractionCreate,
                data: &::nightfall::export::twilight_model::application::interaction::application_command::CommandData,
            ) -> Result<(), ::nightfall::Error> {
                #statements
                Err(::nightfall::Error::CommandNotFound)
            }
        }
    })
}
