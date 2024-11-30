extern crate proc_macro;
mod generate;

use crate::generate::generate_command_controller;
use darling::FromMeta;
use darling::{ast::NestedMeta, FromAttributes};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

pub(crate) trait Error {
    fn write_errors(&self) -> proc_macro2::TokenStream;
}

impl Error for syn::Error {
    fn write_errors(&self) -> proc_macro2::TokenStream {
        self.to_compile_error()
    }
}

impl Error for darling::Error {
    fn write_errors(&self) -> proc_macro2::TokenStream {
        darling::Error::write_errors(self.clone())
    }
}

#[derive(Debug, FromMeta)]
pub(crate) struct CommandControllerConfig {
    sub: Option<String>,
    sub_description: Option<String>,
    group: Option<String>,
}

#[derive(Debug, FromMeta)]
pub(crate) struct ChoiceInfo {
    name: String,
    value: syn::Lit,
}

#[derive(Debug, FromMeta)]
pub(crate) struct OptionInfo {
    name: Option<String>,
    description: String,
    #[darling(multiple, rename = "choice")]
    choices: Vec<ChoiceInfo>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(command))]
pub(crate) struct CommandInfo {
    name: Option<String>,
    description: String,
    #[darling(multiple, rename = "option")]
    options: Vec<OptionInfo>,
    interaction: Option<syn::Path>,
}

#[proc_macro_attribute]
pub fn command(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn command_controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_ = parse_macro_input!(item as ItemImpl);

    let config = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => match CommandControllerConfig::from_list(&v) {
            Ok(c) => c,
            Err(e) => return e.write_errors().into(),
        },
        Err(e) => return e.write_errors().into(),
    };

    if config.group.is_some() && config.sub.is_none() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Cannot specify group alone, sub needs to be specified if group is specified",
        )
        .to_compile_error()
        .into();
    }

    if config.sub.is_some() && config.sub_description.is_none() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "You need to specify sub_description if sub is specified",
        )
        .to_compile_error()
        .into();
    }

    match generate_command_controller(impl_, config) {
        Ok(t) => t.into(),
        Err(e) => e.write_errors().into(),
    }
}
