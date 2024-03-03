extern crate proc_macro;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Ident, LitInt, Meta, Token,
};

struct FileFlowGenerator {
    input: DeriveInput,
    migrations: Option<HashSet<Ident>>,
}

impl FileFlowGenerator {
    pub fn new(input: DeriveInput, migrations: Option<HashSet<Ident>>) -> Self {
        Self { input, migrations }
    }

    pub fn generate_file_flow(&self) -> TokenStream {
        let struct_name = self.input.ident.clone();
        let current_flow_id = Ident::new(
            &gen_flow_id_name(&self.input.ident),
            proc_macro2::Span::call_site(),
        );
        let migrations: Vec<proc_macro2::TokenStream> = self
            .migrations
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|i| {
                let const_flow_id_name =
                    Ident::new(&gen_flow_id_name(&i), proc_macro2::Span::call_site());
                quote! {
                    #const_flow_id_name => E::deserialize::<#i>(&bytes).map(#struct_name::from),
                }
            })
            .collect();

        let file_flow_impl = quote! {
            impl serde_flow::flow::FileFlowRunner<#struct_name> for #struct_name {
                fn load_from_path<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<#struct_name> {
                    if !path.exists() {
                        return Err(serde_flow::error::SerdeFlowError::FileNotFound);
                    }

                    let mut bytes = std::fs::read(path)?;
                    if bytes.len() < 2 {
                        // Handle the case where there are not enough bytes to form a u16
                        panic!("Object does not contain enough bytes");
                    }

                    // Extract the first two bytes and convert them to a u16 in little-endian format
                    let flow_id = u16::from_le_bytes([bytes[0], bytes[1]]);

                    // Remove the first two bytes from the original Vec<u8>
                    let bytes = bytes.split_off(2);
                    match flow_id {
                        #current_flow_id => E::deserialize::<#struct_name>(&bytes),
                        #(#migrations)*
                        _ => Err(serde_flow::error::SerdeFlowError::VariantNotFound),
                    }
                }

                fn save_to_path<E: serde_flow::encoder::FlowEncoder>(&self, path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    let mut flow_id_bytes = #current_flow_id.to_le_bytes().to_vec();
                    let bytes = E::serialize::<#struct_name>(self)?;
                    flow_id_bytes.extend_from_slice(&bytes);
                    std::fs::write(path, flow_id_bytes)?;
                    Ok(())
                }
            }

        };

        if self.migrations.is_none() {
            return file_flow_impl.into();
        }

        let total_impl = quote! {
            #file_flow_impl
            impl serde_flow::flow::FileFlowMigrateRunner<#struct_name> for #struct_name {
                fn load_and_migrate<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<#struct_name> {
                    use serde_flow::flow::FileFlowRunner;
                    let object = #struct_name::load_from_path::<E>(path)?;
                    object.save_to_path::<E>(path)?;
                    Ok(object)
                }
                fn migrate<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    use serde_flow::flow::FileFlowRunner;
                    let object = #struct_name::load_from_path::<E>(path)?;
                    object.save_to_path::<E>(path)?;
                    Ok(())
                }
            }
        };

        total_impl.into()
    }
}

struct AttributeArgs {
    pub args: HashSet<Ident>,
}
impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            args: vars.into_iter().collect(),
        })
    }
}

struct AttributeLitInt {
    pub value: LitInt,
}

impl Parse for AttributeLitInt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            value: input.parse::<LitInt>()?,
        })
    }
}

#[proc_macro_derive(FileFlow, attributes(migrations))]
pub fn file_flow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attrs = input.attrs.clone();
    let migrations = attrs.iter().find(|attr| attr.path().is_ident("migrations"));

    let migrations: Option<HashSet<Ident>> = if let Some(migrations) = migrations {
        let meta = &migrations.meta;
        if let Meta::List(meta_list) = meta {
            let token_stream: TokenStream = meta_list.tokens.clone().into();
            let migrations_args = parse_macro_input!(token_stream as AttributeArgs);
            Some(migrations_args.args)
        } else {
            None
        }
    } else {
        None
    };

    let file_flow_gen = FileFlowGenerator::new(input, migrations);
    file_flow_gen.generate_file_flow()
}

#[proc_macro_derive(FlowVariant, attributes(variant))]
pub fn flow_variant_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attrs = input.attrs.clone();
    let variant = attrs
        .iter()
        .find(|attr| attr.path().is_ident("variant"))
        .expect("variant macro is required");

    let meta = &variant.meta;
    let variant: LitInt = if let Meta::List(meta_list) = meta {
        let token_stream: TokenStream = meta_list.tokens.clone().into();
        let variant = parse_macro_input!(token_stream as AttributeLitInt);
        variant.value
    } else {
        panic!("Failed to decode variant");
    };

    let flow_id_name = Ident::new(
        &gen_flow_id_name(&input.ident),
        proc_macro2::Span::call_site(),
    );
    let flow_variant_impl = quote! {
        const #flow_id_name: u16 = #variant;
    };

    flow_variant_impl.into()
}

fn gen_flow_id_name(iden: &Ident) -> String {
    format!("FLOW_ID_{}", iden.to_string().to_uppercase())
}

// #[proc_macro_derive(FlowId, attributes())]
// pub fn flow_id_derive(_item: TokenStream) -> TokenStream {
//     TokenStream::new()
// }
