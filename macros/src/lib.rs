extern crate proc_macro;
mod generate;

use crate::generate::generate_command_controller;
use darling::ast::NestedMeta;
use darling::{FromAttributes, FromField, FromMeta, FromTypeParam, FromVariant};
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

#[derive(Debug, FromMeta)]
pub(crate) struct CommandControllerConfig {
}

#[derive(Debug, FromMeta)]
pub(crate) struct CommandInfo {

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

    match generate_command_controller(impl_, config) {
        Ok(t) => t.into(),
        Err(e) => e.write_errors().into(),
    }
}
