extern crate proc_macro;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
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
        let current_flow_id = gen_flow_id_name(&self.input.ident);
        let flow_id_struct_name = gen_flow_id_struct_name(&self.input.ident);
        let migrations: Vec<proc_macro2::TokenStream> = self
            .migrations
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|i| {
                let const_flow_id_name = gen_flow_id_name(&i);
                let flow_id_vs = gen_flow_id_struct_name(&i);
                quote! {
                    #const_flow_id_name => E::deserialize::<#flow_id_vs>(&bytes).map(#i::from).map(#struct_name::from),
                }
            })
            .collect();

        let file_flow_impl = quote! {
            impl serde_flow::flow::File<#struct_name> for #struct_name {
                fn load_from_path<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<#struct_name> {
                    if !path.exists() {
                        return Err(serde_flow::error::SerdeFlowError::FileNotFound);
                    }

                    let mut bytes = std::fs::read(path)?;
                    if bytes.len() < 2 {
                        return Err(serde_flow::error::SerdeFlowError::FormatInvalid);
                    }
                    let flow_id_object = E::deserialize::<serde_flow::flow::FlowId>(&bytes)?;
                    match flow_id_object.flow_id {
                        #current_flow_id => E::deserialize::<#flow_id_struct_name>(&bytes).map(#struct_name::from),
                        #(#migrations)*
                        _ => Err(serde_flow::error::SerdeFlowError::VariantNotFound),
                    }
                }

                fn save_to_path<E: serde_flow::encoder::FlowEncoder>(&self, path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    let mut flow_object = #flow_id_struct_name::new(#current_flow_id, self);
                    let bytes = E::serialize::<#flow_id_struct_name>(&flow_object)?;
                    std::fs::write(path, &bytes)?;
                    Ok(())
                }
            }

        };

        if self.migrations.is_none() {
            return file_flow_impl.into();
        }

        let total_impl = quote! {
            #file_flow_impl
            impl serde_flow::flow::FileMigrate<#struct_name> for #struct_name {
                fn load_and_migrate<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<#struct_name> {
                    use serde_flow::flow::File;
                    let object = #struct_name::load_from_path::<E>(path)?;
                    object.save_to_path::<E>(path)?;
                    Ok(object)
                }
                fn migrate<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    use serde_flow::flow::File;
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
    let fields_gen = FieldsGenerator::parse(input.clone());
    let fields = fields_gen.fields();
    let field_names = fields_gen.field_names();

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
        panic!("This macro only supports structs");
    };

    let struct_name = input.ident.clone();
    let flow_id_name = gen_flow_id_name(&input.ident);
    let flow_id_struct_name = gen_flow_id_struct_name(&input.ident);
    let flow_variant_impl = quote! {
        const #flow_id_name: u16 = #variant;
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct #flow_id_struct_name {
            pub flow_id: u16,
            #(#fields,)*
        }

        impl #flow_id_struct_name {
            pub fn new(flow_id: u16, item: &#struct_name) -> Self {
                Self {
                    flow_id,
                    #(#field_names: item.#field_names.clone(),)*
                }
            }
        }

        impl From<#flow_id_struct_name> for #struct_name {
            fn from(item: #flow_id_struct_name) -> Self {
                #struct_name {
                    #(#field_names: item.#field_names,)*
                }
            }
        }

    };

    flow_variant_impl.into()
}

struct FieldsGenerator {
    fields: syn::FieldsNamed,
}

impl FieldsGenerator {
    pub fn parse(input: DeriveInput) -> Self {
        let fields: syn::FieldsNamed = if let syn::Data::Struct(s) = &input.data {
            match s.fields.clone() {
                syn::Fields::Named(fields) => fields,
                _ => panic!("Unit structs are not supported"),
            }
        } else {
            panic!("This macro only supports structs");
        };
        Self { fields }
    }

    pub fn fields(&self) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .named
            .iter()
            .map(|f| f.to_token_stream())
            .collect()
    }

    pub fn field_names(&self) -> Vec<Ident> {
        self.fields
            .named
            .iter()
            .map(|f| f.ident.clone().unwrap())
            .collect()
    }
}

fn gen_flow_id_name(iden: &Ident) -> Ident {
    Ident::new(
        &format!("FLOW_ID_{}", iden.to_string().to_uppercase()),
        proc_macro2::Span::call_site(),
    )
}

fn gen_flow_id_struct_name(ident: &Ident) -> Ident {
    Ident::new(&format!("{ident}FlowId"), proc_macro2::Span::call_site())
}
