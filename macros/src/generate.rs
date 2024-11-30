use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use std::ops::Deref;
use syn::spanned::Spanned;
use syn::{ImplItem, ImplItemFn, ItemImpl};

fn get_inner_optional(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };

    if !path.path.is_ident("Option") {
        return None;
    };

    let segment = path.path.segments.first().unwrap();
    let syn::PathArguments::AngleBracketed(bracket) = &segment.arguments else {
        return None;
    };

    bracket.args.first().and_then(|i| {
        if let syn::GenericArgument::Type(t) = i {
            Some(t.clone())
        } else {
            None
        }
    })
}

fn get_arg(
    args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    index: usize,
    offset: &mut usize,
    interaction_name: &String,
) -> Option<syn::FnArg> {
    let arg = match args.get(index + *offset) {
        Some(a) => a,
        None => return None,
    };

    let typed = match arg {
        syn::FnArg::Receiver(_) => {
            *offset += 1;
            return get_arg(args, index, offset, interaction_name);
        }
        syn::FnArg::Typed(t) => t,
    };

    let syn::Pat::Ident(ident) = typed.pat.deref() else {
        *offset += 1;
        return get_arg(args, index, offset, interaction_name);
    };

    if &ident.ident.to_string() == interaction_name {
        *offset += 1;
        return get_arg(args, index, offset, interaction_name);
    }

    Some(arg.clone())
}

fn generate_register_command(
    impl_: &ItemImpl,
    args: &crate::CommandControllerConfig,
) -> Result<TokenStream, Box<dyn crate::Error>> {
    let fn_items: Vec<&ImplItemFn> = impl_
        .items
        .iter()
        .filter_map(|m| match m {
            ImplItem::Fn(i) => Some(i),
            _ => None,
        })
        .collect();

    let mut command_streams = vec![];
    for fn_item in fn_items {
        let info = match crate::CommandInfo::from_attributes(&fn_item.attrs) {
            Ok(a) => a,
            Err(e) => return Err(Box::new(e)),
        };

        let interaction_param = info
            .interaction
            .as_ref()
            .and_then(|i| i.get_ident())
            .map(|i| i.to_string())
            .unwrap_or_else(|| String::from("interaction"));

        let mut options = vec![];
        let mut offset = 0;
        for (i, option) in info.options.into_iter().enumerate() {
            let arg = match get_arg(&fn_item.sig.inputs, i, &mut offset, &interaction_param) {
                Some(a) => a,
                None => {
                    return Err(Box::new(syn::Error::new(
                        fn_item.span(),
                        "More options than there are arguments",
                    )))
                }
            };

            let mut choices = vec![];
            for choice in option.choices {
                let name = choice.name;
                let value = choice.value;
                choices.push(quote! {(#name.into(), #value.into())})
            }

            let syn::FnArg::Typed(typed) = arg else {
                todo!();
            };

            let choice_vec = quote! { vec![#(#choices),*] };
            let syn::Pat::Ident(ident) = typed.pat.deref() else {
                todo!();
            };

            let name = option.name.clone().unwrap_or(ident.ident.to_string());
            let description = &option.description;

            let inner_optional = get_inner_optional(&typed.ty);
            let stream = if let Some(t) = inner_optional {
                quote! {
                    <#t as CreateOption>::create_option(#name, #description, true)
                }
            } else {
                let ty = &typed.ty;
                quote! {
                    <#ty as ::nightfall::register::CreateOption>::create_option(#name, #description, true, #choice_vec)
                }
            };

            options.push(stream);
        }

        let name = info.name.unwrap_or(fn_item.sig.ident.to_string());
        let description = info.description;

        let command = if args.sub.is_some() {
            quote! {
                ::nightfall::export::twilight_util::builder::command::SubCommandBuilder::new(
                    #name,
                    #description,
                )
                #(.option(#options))*
                .build()
            }
        } else {
            quote! {
                ::nightfall::export::twilight_util::builder::command::CommandBuilder::new(
                    #name,
                    #description,
                    ::nightfall::export::twilight_model::application::command::CommandType::ChatInput,
                )
                #(.option(#options))*
                .build()
            }
        };
        command_streams.push(command);
    }

    let commands = if let Some(sub) = args.sub.as_ref() {
        let sub_description = args.sub_description.as_ref().unwrap();

        quote! {
            vec![::nightfall::export::twilight_util::builder::command::CommandBuilder::new(
                #sub,
                #sub_description,
                ::nightfall::export::twilight_model::application::command::CommandType::ChatInput,
            )
            #(.option(#command_streams))*
            .build()]
        }
    } else {
        quote! {
            vec![#(#command_streams),*]
        }
    };
    Ok(commands)
}

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

    let mut command_names = vec![];
    let mut statements = quote! {};
    let options_var = if args.sub.is_some() {
        quote! { sub_options }
    } else {
        quote! { data.options }
    };
    let name_var = if args.sub.is_some() {
        quote! { sub_name }
    } else {
        quote! { data.name }
    };

    for fn_item in fn_items {
        let ident = &fn_item.sig.ident;
        let name = fn_item.sig.ident.to_string();

        let mut args: Vec<TokenStream> = vec![];
        let mut is_self = false;
        let mut offset = 0;
        for (i, arg) in fn_item.sig.inputs.iter().enumerate() {
            let info = match crate::CommandInfo::from_attributes(&fn_item.attrs) {
                Ok(a) => a,
                Err(e) => return Err(Box::new(e)),
            };

            let interaction_param = info
                .interaction
                .as_ref()
                .and_then(|i| i.get_ident())
                .map(|i| i.to_string())
                .unwrap_or_else(|| String::from("interaction"));

            let ty = match arg {
                syn::FnArg::Typed(t) => t,
                syn::FnArg::Receiver(r) => {
                    if r.mutability.is_some() {
                        return Err(Box::new(syn::Error::new(
                            arg.span(),
                            "Self must be non mutable reference", // Dependency injection only gives us a non mutable arc
                        )));
                    } else if r.reference.is_none() {
                        return Err(Box::new(syn::Error::new(
                            arg.span(),
                            "Self must be reference", // We can not move it cause it is a part of a Arc that is managed by the dependency injection
                        )));
                    }

                    is_self = true;
                    offset += 1;
                    continue;
                }
            };

            let arg_name = if let syn::Pat::Ident(i) = ty.pat.deref() {
                i.ident.to_string()
            } else {
                return Err(Box::new(syn::Error::new(
                    ty.span(),
                    "Unrecognised identifier",
                )));
            };

            if arg_name == interaction_param {
                offset += 1;
                args.push(quote! { interaction })
            } else {
                let Some(opt_info) = info.options.get(i - offset) else {
                    return Err(Box::new(syn::Error::new(
                        fn_item.span(),
                        "More options than there are arguments",
                    )));
                };
                let arg_name = opt_info.name.as_ref().unwrap_or(&arg_name);

                args.push(quote! {
                    match #options_var.iter().find(|o| o.name == #arg_name) {
                        Some(v) => match ::nightfall::FromOption::from_option(v.value.clone()) {
                            Some(v2) => v2,
                            None => return Err(::nightfall::Error::OptionBindingFailed)
                        },
                        None => return Err(::nightfall::Error::OptionBindingFailed),
                    }
                });
            }
        }

        command_names.push(ident.to_string());
        let call = if is_self {
            quote! { self.#ident(#(#args),*).await }
        } else {
            quote! { Self::#ident(#(#args),*).await }
        };

        statements = quote! {
            #statements
            if #name_var == #name {
                return match #call {
                    Ok(()) => Ok(()),
                    Err(e) => Err(::nightfall::Error::CommandError { error: e }),
                };
            }
        }
    }

    let register = generate_register_command(&impl_, &args)?;

    let get_command_names = if let Some(sub) = args.sub.as_ref() {
        quote! {
            fn get_command_names<'a>() -> &'a[&'static str] {
                &[#sub]
            }
        }
    } else {
        quote! {
            fn get_command_names<'a>() -> &'a[&'static str] {
                &[#(#command_names),*]
            }
        }
    };

    let execute_command = if let Some(sub) = args.sub.as_ref() {
        quote! {
            if data.name != #sub {
                return Err(::nightfall::Error::CommandNotFound);
            }

            let Some(sub) = data.options.first() else {
                return Err(::nightfall::Error::CommandNotFound);
            };

            let sub_name = sub.name.clone();
            let ::nightfall::export::twilight_model::application::interaction::application_command::CommandOptionValue::SubCommand(sub_options)
                = &sub.value else {
                return Err(::nightfall::Error::OptionBindingFailed)
            };

            #statements
            Err(::nightfall::Error::CommandNotFound)
        }
    } else {
        quote! { 
            #statements
            Err(::nightfall::Error::CommandNotFound)
        }
    };

    Ok(quote! {
        #impl_

        #[::nightfall::export::async_trait::async_trait]
        impl ::nightfall::CommandController for #struct_name {
            async fn execute_command(
                &self,
                interaction: &::nightfall::export::twilight_model::gateway::payload::incoming::InteractionCreate,
                data: &::nightfall::export::twilight_model::application::interaction::application_command::CommandData,
            ) -> Result<(), ::nightfall::Error> {
                #execute_command
            }

            #get_command_names

            fn build_commands() -> Vec<::nightfall::export::twilight_model::application::command::Command> {
                #register
            }
        }
    })
}
