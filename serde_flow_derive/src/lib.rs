extern crate proc_macro;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    DeriveInput, Ident, LitInt, Meta, MetaNameValue, Token,
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

                    let bytes = std::fs::read(path)?;
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
                    object.save_to_path::<E>(path)
                }
            }
        };

        total_impl.into()
    }

    pub fn generate_file_flow_zerocopy(&self) -> TokenStream {
        let struct_name = self.input.ident.clone();
        let current_flow_id = gen_flow_id_name(&self.input.ident);
        let migrations: Vec<proc_macro2::TokenStream> = self
            .migrations
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|i| {
                let const_flow_id_name = gen_flow_id_name(&i);
                quote! {
                    #const_flow_id_name => {
                        use serde_flow::flow::zerocopy::File;
                        let old_object = serde_flow::encoder::zerocopy::Reader::<#i>::new(bytes).deserialize()?;
                        let converted = #struct_name::from(old_object);
                        converted.save_to_path(path)?;
                        #struct_name::from_path(path)
                    },
                }
            })
            .collect();

        let file_flow_impl = quote! {
            impl serde_flow::flow::zerocopy::File<#struct_name> for #struct_name {
                fn from_path(path: &std::path::Path) -> serde_flow::flow::FlowResult<serde_flow::encoder::zerocopy::Reader<#struct_name>> {
                    if !path.exists() {
                        return Err(serde_flow::error::SerdeFlowError::FileNotFound);
                    }

                    let mut bytes = std::fs::read(path)?;
                    if bytes.len() < 2 {
                        return Err(serde_flow::error::SerdeFlowError::FormatInvalid);
                    }
                    // Extract the first two bytes and convert them to a u16 in little-endian format
                    let flow_id = u16::from_le_bytes([bytes[0], bytes[1]]);

                    // Remove the first two bytes from the original Vec<u8>
                    let bytes = bytes.split_off(2);
                    match flow_id {
                        #current_flow_id => Ok(serde_flow::encoder::zerocopy::Reader::<#struct_name>::new(bytes)),
                        #(#migrations)*
                        _ => Err(serde_flow::error::SerdeFlowError::VariantNotFound),
                    }
                }

                fn save_to_path(&self, path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    let mut total_bytes = #current_flow_id.to_le_bytes().to_vec();
                    let bytes = serde_flow::encoder::zerocopy::Encoder::serialize::<#struct_name>(self)?;
                    total_bytes.extend_from_slice(&bytes);
                    std::fs::write(path, &total_bytes)?;
                    Ok(())
                }
            }

        };

        if self.migrations.is_none() {
            return file_flow_impl.into();
        }

        let total_impl = quote! {
            #file_flow_impl
            impl serde_flow::flow::zerocopy::FileMigrate<#struct_name> for #struct_name {
                fn load_and_migrate(path: &std::path::Path)
                    -> serde_flow::flow::FlowResult<serde_flow::encoder::zerocopy::Reader<#struct_name>>
                {
                    use serde_flow::flow::zerocopy::File;
                    #struct_name::from_path(path)
                }
                fn migrate(path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    use serde_flow::flow::zerocopy::File;
                    let _ = #struct_name::from_path(path)?;
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

#[proc_macro_derive(FileFlowZeroCopy, attributes(migrations))]
pub fn file_flow_zerocopy_derive(input: TokenStream) -> TokenStream {
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
    file_flow_gen.generate_file_flow_zerocopy()
}

#[proc_macro_derive(FlowVariant, attributes(variant, zerocopy))]
pub fn flow_variant_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields_gen = FieldsGenerator::parse(&input);
    let fields = fields_gen.fields();
    let field_names = fields_gen.field_names();

    let attrs = input.attrs.clone();
    let variant = attrs
        .iter()
        .find(|attr| attr.path().is_ident("variant"))
        .expect("variant macro is required");

    let is_zerocopy = attrs.iter().any(|attr| attr.path().is_ident("zerocopy"));

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
    let flow_variant_const_impl = quote! {
        const #flow_id_name: u16 = #variant;
    };

    if is_zerocopy {
        return flow_variant_const_impl.into();
    }

    let flow_variant_impl = quote! {
        #flow_variant_const_impl

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

#[proc_macro_derive(Flow, attributes(flow, variants))]
pub fn flow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields_gen = FieldsGenerator::parse(&input);
    let fields = fields_gen.fields();
    let field_names = fields_gen.field_names();

    let attrs = input.attrs.clone();
    let flow = attrs
        .iter()
        .find(|attr| attr.path().is_ident("flow"))
        .expect("flow macro is required");
    let variants_attr = attrs
        .iter()
        .find(|attr| attr.path().is_ident("variants"))
        .expect("variants macro is required");

    // Extract meta items from the attribute
    let mut version = None;
    let mut is_file = false;
    let mut is_bytes = false;
    let mut is_nonbloking = false;
    let mut is_bloking = false;
    let mut is_zerocopy = false;
    let mut variants = Vec::new();

    match &flow.meta {
        syn::Meta::List(meta_list) => meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident("variant") {
                let value = meta.value().unwrap(); // this parses the `=`
                let lit: syn::LitInt = value.parse().unwrap();
                version = Some(lit.base10_parse::<u16>().unwrap());
                return Ok(());
            }

            if meta.path.is_ident("bytes") {
                is_bytes = true;
                return Ok(());
            }

            if meta.path.is_ident("file") {
                is_file = true;
                if meta.input.peek(syn::token::Paren) {
                    meta.parse_nested_meta(|file_meta| {
                        if file_meta.path.is_ident("blocking") {
                            is_nonbloking = true;
                            return Ok(());
                        }
                        if file_meta.path.is_ident("nonbloking") {
                            is_nonbloking = true;
                            return Ok(());
                        }
                        if file_meta.path.is_ident("zerocopy") {
                            is_zerocopy = true;
                            return Ok(());
                        }
                        Err(file_meta.error("unsupported file property"))
                    });
                } else {
                    is_bloking = true;
                }
                return Ok(());
            }

            Err(meta.error("unsupported flow property"))
        }),
        _ => return TokenStream::new(),
    };

    match &variants_attr.meta {
        syn::Meta::List(meta_list) => meta_list.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident().cloned() {
                variants.push(ident);
                return Ok(());
            }
            Err(meta.error("unsupported variants property"))
        }),
        _ => return TokenStream::new(),
    };

    println!(
        "version\t{version:?}\nis_file:\t{is_file}\nnonbloking:\t{is_nonbloking}\nis_zerocopy:\t{is_zerocopy}",
    );

    TokenStream::new()
}

#[derive(Default)]
struct FlowGenerator {
    version: Option<u16>,
    is_file: bool,
    is_bytes: bool,
    is_nonbloking: bool,
    is_bloking: bool,
    is_zerocopy: bool,
    variants: Option<Vec<Ident>>,
    fields_gen: Option<FieldsGenerator>,
}

impl FlowGenerator {
    pub fn parse(input: DeriveInput) -> syn::parse::Result<Self> {
        let mut flow_gen = FlowGenerator::default();
        let attrs = input.attrs.clone();
        let flow_attr = attrs
            .iter()
            .find(|attr| attr.path().is_ident("flow"))
            .ok_or(syn::parse::Error::new(
                input.span(),
                "flow attribute must be provided",
            ))?;
        let variants_attr = attrs.iter().find(|attr| attr.path().is_ident("variants"));

        // parse #flow attribute
        flow_gen.parse_flow(flow_attr)?;

        // parse #variants attribute
        if let Some(variants_attr) = variants_attr {
            flow_gen.parse_variants(variants_attr)?;
        }

        // parse structure's fields
        let fields_gen = FieldsGenerator::parse(&input);
        flow_gen.fields_gen = Some(fields_gen);

        Ok(flow_gen)
    }

    fn parse_flow(&mut self, attr: &syn::Attribute) -> syn::parse::Result<()> {
        match &attr.meta {
            syn::Meta::List(meta_list) => meta_list.parse_nested_meta(|meta| {
                if meta.path.is_ident("variant") {
                    let value = meta.value().unwrap(); // this parses the `=`
                    let lit: syn::LitInt = value.parse().unwrap();
                    self.version = Some(lit.base10_parse::<u16>().unwrap());
                    return Ok(());
                }

                if meta.path.is_ident("bytes") {
                    self.is_bytes = true;
                    return Ok(());
                }

                if meta.path.is_ident("file") {
                    self.is_file = true;
                    if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|file_meta| {
                            if file_meta.path.is_ident("blocking") {
                                self.is_nonbloking = true;
                                return Ok(());
                            }
                            if file_meta.path.is_ident("nonbloking") {
                                self.is_nonbloking = true;
                                return Ok(());
                            }
                            if file_meta.path.is_ident("zerocopy") {
                                self.is_zerocopy = true;
                                return Ok(());
                            }
                            Err(file_meta.error("unsupported file property"))
                        });
                    } else {
                        self.is_bloking = true;
                    }
                    return Ok(());
                }

                Err(meta.error("unsupported flow property"))
            }),
            _ => Err(syn::parse::Error::new(
                attr.span(),
                "Failed to parse flow attribute",
            )),
        }
    }

    fn parse_variants(&mut self, attr: &syn::Attribute) -> syn::parse::Result<()> {
        let mut variants = Vec::new();
        let result = match &attr.meta {
            syn::Meta::List(meta_list) => meta_list.parse_nested_meta(|meta| {
                if let Some(ident) = meta.path.get_ident().cloned() {
                    variants.push(ident);
                    return Ok(());
                }
                Err(meta.error("unsupported variants property"))
            }),
            _ => Err(syn::parse::Error::new(
                attr.span(),
                "Failed to parse vartiants",
            )),
        };
        if result.is_ok() {
            self.variants = Some(variants);
        }
        result
    }
}

struct FieldsGenerator {
    fields: syn::FieldsNamed,
}

impl FieldsGenerator {
    pub fn parse(input: &DeriveInput) -> Self {
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
