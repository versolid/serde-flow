extern crate proc_macro;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Ident, Meta, Token,
};

struct FileFlowGenerator {
    input: DeriveInput,
    variants: Option<HashSet<Ident>>,
}

impl FileFlowGenerator {
    pub fn new(input: DeriveInput, variants: Option<HashSet<Ident>>) -> Self {
        Self { input, variants }
    }

    pub fn generate_file_flow(&self) -> TokenStream {
        let struct_name = self.input.ident.clone();
        let variants: Vec<Ident> = self
            .variants
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect();

        let file_flow_impl = quote! {
            use serde_flow::{flow::{FileFlow, FileFlowMigrate, FlowResult}, encoder::FlowEncoder, error::SerdeFlowError};
            impl FileFlow<#struct_name> for #struct_name {
                fn load_from_path<E: FlowEncoder>(path: &std::path::Path) -> FlowResult<#struct_name> {
                    if !path.exists() {
                        return Err(SerdeFlowError::FileNotFound);
                    }
                    let bytes = std::fs::read(path)?;

                    if let Ok(object) = E::deserialize::<#struct_name>(&bytes) {
                        return Ok(object);
                    }
                    #(
                        if let Ok(variant) = E::deserialize::<#variants>(&bytes) {
                            return Ok(#struct_name::from(variant));
                        }
                    )*
                    return Err(SerdeFlowError::ParsingFailed);
                }

                fn save_on_path<E: FlowEncoder>(&self, path: &std::path::Path) -> FlowResult<()> {
                    let bytes = E::serialize::<#struct_name>(self)?;
                    std::fs::write(path, bytes)?;
                    Ok(())
                }
            }
        };

        if self.variants.is_none() {
            return file_flow_impl.into();
        }

        let total_impl = quote! {
            #file_flow_impl

            impl FileFlowMigrate<#struct_name> for #struct_name {
                fn load_and_migrate<E: FlowEncoder>(path: &std::path::Path) -> FlowResult<#struct_name> {
                    use serde_flow::FileFlow;
                    let object = #struct_name::load_from_path::<E>(path)?;
                    object.save_on_path::<E>(path);
                    Ok(object)
                }
                fn migrate<E: FlowEncoder>(path: &std::path::Path) -> FlowResult<()> {
                    use serde_flow::FileFlow;
                    let object = #struct_name::load_from_path::<E>(path)?;
                    object.save_on_path::<E>(path);
                    Ok(())
                }
            }
        };

        total_impl.into()
    }
}

struct VariantArgs {
    pub variants: HashSet<Ident>,
}
impl Parse for VariantArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            variants: vars.into_iter().collect(),
        })
    }
}

#[proc_macro_derive(FileFlow, attributes(variant))]
pub fn file_flow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attrs = input.attrs.clone();
    let variants = attrs.iter().find(|attr| attr.path().is_ident("variant"));

    let variants: Option<HashSet<Ident>> = if let Some(variants) = variants {
        let meta = &variants.meta;
        if let Meta::List(meta_list) = meta {
            let token_stream: TokenStream = meta_list.tokens.clone().into();
            let variants_args = parse_macro_input!(token_stream as VariantArgs);
            Some(variants_args.variants)
        } else {
            None
        }
    } else {
        None
    };

    let file_flow_gen = FileFlowGenerator::new(input, variants);
    file_flow_gen.generate_file_flow()
}
