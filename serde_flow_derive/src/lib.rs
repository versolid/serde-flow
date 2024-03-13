extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident};

#[proc_macro_derive(Flow, attributes(flow, variants))]
pub fn flow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // parsing flow generator
    let flow_gen = match FlowGenerator::parse(input) {
        Ok(gen) => gen,
        Err(e) => {
            eprintln!("{e}");
            return TokenStream::new();
        }
    };

    let flow = flow_gen.generate();
    flow.into()
}

struct FlowGenerator {
    struct_name: Ident,
    variant: u16,
    is_file: bool,
    is_bytes: bool,
    is_nonbloking: bool,
    is_bloking: bool,
    is_zerocopy: bool,
    is_verify_write: bool,
    variants: Option<Vec<Ident>>,
    fields_gen: FieldsGenerator,
}

impl FlowGenerator {
    fn generate(&self) -> proc_macro2::TokenStream {
        let previous = self.generate_ids();
        let previous = self.generate_bytes(previous);
        self.generate_file(previous)
    }

    fn generate_ids(&self) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();
        let variant = self.variant;
        let flow_id_name = gen_variant_id_name(&struct_name);
        let flow_variant_const_impl = quote! {
            const #flow_id_name: u16 = #variant;
        };

        if self.is_zerocopy {
            return flow_variant_const_impl;
        }

        // prepare transformers for non zerocopy
        let flow_id_struct_name = gen_variant_dto_name(&struct_name);
        let fields = self.fields_gen.fields();
        let field_names = self.fields_gen.field_names();
        quote! {
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
        }
    }

    fn generate_bytes(&self, previous: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if !self.is_bytes {
            return previous;
        }

        let struct_name = self.struct_name.clone();
        let encode_with_version = self.encode_with_version();
        let decode_with_version = self.decode_with_version();

        if self.is_zerocopy {
            return quote! {
                #previous
                impl serde_flow::flow::zerocopy::Bytes<#struct_name> for #struct_name {
                    fn encode(&self) -> serde_flow::flow::FlowResult<Vec<u8>> {
                        #encode_with_version
                        Ok(total_bytes)
                    }
                    fn decode(bytes: Vec<u8>) -> serde_flow::flow::FlowResult<Reader<#struct_name>> {
                        #decode_with_version
                    }
                }
            };
        }
        quote! {
            #previous
            impl serde_flow::flow::Bytes<#struct_name> for #struct_name {
                fn encode<E: serde_flow::encoder::FlowEncoder>(&self) -> serde_flow::flow::FlowResult<Vec<u8>> {
                    #encode_with_version
                    Ok(total_bytes)
                }
                fn decode<E: serde_flow::encoder::FlowEncoder>(bytes: &[u8]) -> serde_flow::flow::FlowResult<#struct_name> {
                    #decode_with_version
                }
            }
        }
    }

    fn generate_file(&self, previous: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if !self.is_file {
            return previous;
        }

        if self.is_zerocopy {
            self.generate_zerocopy_file(previous)
        } else {
            self.generate_normal_file(previous)
        }
    }

    fn generate_zerocopy_file(
        &self,
        previous: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();

        let mut generated = previous;
        if self.is_bloking {
            let func_load_from_path = self.component_load_from_path(self.is_zerocopy, true);
            let func_save_to_path = self.component_save_to_path(self.is_zerocopy, true);
            generated = quote! {
                #generated
                impl serde_flow::flow::zerocopy::File<#struct_name> for #struct_name {
                    #func_load_from_path
                    #func_save_to_path
                }
            };
        }

        if self.is_nonbloking {
            let func_load_from_path = self.component_load_from_path(self.is_zerocopy, false);
            let func_save_to_path = self.component_save_to_path(self.is_zerocopy, false);
            generated = quote! {
                #generated
                impl serde_flow::flow::zerocopy::FileAsync<#struct_name> for #struct_name {
                    #func_load_from_path
                    #func_save_to_path
                }
            };
        }

        if self.variants.is_none() {
            return generated;
        }

        if self.is_bloking {
            generated = quote! {
                #generated
                impl serde_flow::flow::zerocopy::FileMigrate<#struct_name> for #struct_name {
                    fn load_and_migrate(path: &std::path::Path)
                        -> serde_flow::flow::FlowResult<serde_flow::encoder::zerocopy::Reader<#struct_name>>
                    {
                        use serde_flow::flow::zerocopy::File;
                        #struct_name::load_from_path(path)
                    }
                    fn migrate(path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                        use serde_flow::flow::zerocopy::File;
                        let _ = #struct_name::load_from_path(path)?;
                        Ok(())
                    }
                }
            };
        }

        if self.is_nonbloking {
            generated = quote! {
                #generated
                impl serde_flow::flow::zerocopy::FileMigrateAsync<#struct_name> for #struct_name {
                    fn load_and_migrate_async(path: &std::path::Path)
                        -> serde_flow::flow::AsyncResult<serde_flow::encoder::zerocopy::Reader<#struct_name>>
                    {
                        use serde_flow::flow::zerocopy::FileAsync;
                        #struct_name::load_from_path_async(path)
                    }
                    fn migrate_async(path: &std::path::Path) -> serde_flow::flow::AsyncResult<()> {
                        std::boxed::Box::pin(async {
                            use serde_flow::flow::zerocopy::FileAsync;
                            let _ = #struct_name::load_from_path_async(path).await?;
                            Ok(())
                        })
                    }
                }
            };
        }
        generated
    }

    fn generate_normal_file(&self, previous: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();

        let mut generated = previous;
        if self.is_bloking {
            let func_load_from_path = self.component_load_from_path(self.is_zerocopy, true);
            let func_save_to_path = self.component_save_to_path(self.is_zerocopy, true);
            generated = quote! {
                #generated
                impl serde_flow::flow::File<#struct_name> for #struct_name {
                    #func_load_from_path
                    #func_save_to_path
                }
            };
        }

        if self.is_nonbloking {
            let func_load_from_path = self.component_load_from_path(self.is_zerocopy, false);
            let func_save_to_path = self.component_save_to_path(self.is_zerocopy, false);
            generated = quote! {
                #generated
                impl serde_flow::flow::FileAsync<#struct_name> for #struct_name {
                    #func_load_from_path
                    #func_save_to_path
                }
            };
        }

        if self.variants.is_none() {
            return generated;
        }

        // Migrations
        if self.is_bloking {
            generated = quote! {
                #generated
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
        }

        if self.is_nonbloking {
            generated = quote! {
                #generated
                impl serde_flow::flow::FileMigrateAsync<#struct_name> for #struct_name {
                    fn load_and_migrate_async<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::AsyncResult<#struct_name> {
                        std::boxed::Box::pin(async {
                            use serde_flow::flow::FileAsync;
                            let object = #struct_name::load_from_path_async::<E>(path).await?;
                            object.save_to_path_async::<E>(path).await?;
                            Ok(object)
                        })
                    }
                    fn migrate_async<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::AsyncResult<()> {
                        std::boxed::Box::pin(async {
                            use serde_flow::flow::FileAsync;
                            let object = #struct_name::load_from_path_async::<E>(path).await?;
                            object.save_to_path_async::<E>(path).await
                        })
                    }
                }
            };
        }
        generated
    }

    fn component_load_from_path(
        &self,
        is_zerocopy: bool,
        is_bloking: bool,
    ) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();
        let file_read = Self::component_fs_read(is_bloking);
        let decode_with_version = self.decode_with_version();

        // NON zerocopy
        if !is_zerocopy {
            let func_body = quote! {
                if !path.exists() {
                    return Err(serde_flow::error::SerdeFlowError::FileNotFound);
                }

                #file_read
                #decode_with_version
            };
            if is_bloking {
                return quote! {
                    fn load_from_path<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::FlowResult<#struct_name> {
                        #func_body
                    }
                };
            }
            return quote! {
                fn load_from_path_async<E: serde_flow::encoder::FlowEncoder>(path: &std::path::Path) -> serde_flow::flow::AsyncResult<#struct_name> {
                    std::boxed::Box::pin(async move { #func_body })
                }
            };
        }

        // zerocopy
        let func_body = quote! {
            if !path.exists() {
                return Err(serde_flow::error::SerdeFlowError::FileNotFound);
            }
            #file_read
            #decode_with_version
        };
        if is_bloking {
            quote! {
                fn load_from_path(path: &std::path::Path) -> serde_flow::flow::FlowResult<serde_flow::encoder::zerocopy::Reader<#struct_name>> {
                    #func_body
                }
            }
        } else {
            quote! {
                fn load_from_path_async<'a>(path_to: std::path::PathBuf) -> serde_flow::flow::AsyncResult<'a, serde_flow::encoder::zerocopy::Reader<'a, #struct_name>> {
                    std::boxed::Box::pin(async move {
                        let path = path_to.as_path();
                        #func_body
                    })
                }
            }
        }
    }

    fn component_save_to_path(
        &self,
        is_zerocopy: bool,
        is_bloking: bool,
    ) -> proc_macro2::TokenStream {
        let try_verify_write = self.component_verify_write(is_bloking);

        let encode_body = self.encode_with_version();
        let func_body = quote! {
            #encode_body
            #try_verify_write
        };

        if !is_zerocopy {
            if is_bloking {
                return quote! {
                    fn save_to_path<E: serde_flow::encoder::FlowEncoder>(&self, path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                        #func_body
                    }
                };
            }

            return quote! {
                fn save_to_path_async<'a, E: serde_flow::encoder::FlowEncoder>(&'a self, path: &'a std::path::Path) -> serde_flow::flow::AsyncResult<()> {
                    std::boxed::Box::pin(async move { #func_body })
                }
            };
        }

        if is_bloking {
            quote! {
                fn save_to_path(&self, path: &std::path::Path) -> serde_flow::flow::FlowResult<()> {
                    #func_body
                }
            }
        } else {
            quote! {
                fn save_to_path_async(&self, path_to: std::path::PathBuf) -> serde_flow::flow::AsyncResult<()> {
                    std::boxed::Box::pin(async move {
                        let path = path_to.as_path();
                        #func_body
                    })
                }
            }
        }
    }

    fn encode_with_version(&self) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();
        let current_variant = self.variant;
        if self.is_zerocopy {
            return quote! {
                let mut total_bytes = #current_variant.to_le_bytes().to_vec();
                let bytes = serde_flow::encoder::zerocopy::Encoder::serialize::<#struct_name>(self)?;
                total_bytes.extend_from_slice(&bytes);
            };
        }

        // Normal - NON ZeroCopy
        let current_flow_id = gen_variant_id_name(&struct_name);
        let current_dto_name = gen_variant_dto_name(&struct_name);
        quote! {
            let mut flow_object = #current_dto_name::new(#current_flow_id, self);
            let total_bytes = E::serialize::<#current_dto_name>(&flow_object)?;
        }
    }

    fn decode_with_version(&self) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name.clone();
        let current_variant = self.variant;

        if self.is_zerocopy {
            let variants = self.variants.clone().unwrap_or_default();
            let variants: Vec<proc_macro2::TokenStream> = variants
                .into_iter()
                .map(|i| {
                    let const_variant_id_name = gen_variant_id_name(&i);
                    quote! {
                        #const_variant_id_name => {
                            use serde_flow::flow::zerocopy::File;
                            let old_object = serde_flow::encoder::zerocopy::Reader::<#i>::new(bytes).deserialize()?;
                            let converted = #struct_name::from(old_object);
                            converted.save_to_path(path)?;
                            #struct_name::load_from_path(path)
                        },
                    }
                })
                .collect();

            return quote! {
                if bytes.len() < 2 {
                    return Err(serde_flow::error::SerdeFlowError::FormatInvalid);
                }
                // Extract the first two bytes and convert them to a u16 in little-endian format
                let flow_id = u16::from_le_bytes([bytes[0], bytes[1]]);

                // Remove the first two bytes from the original Vec<u8>
                let bytes = bytes.split_off(2);
                match flow_id {
                    #current_variant => Ok(serde_flow::encoder::zerocopy::Reader::<#struct_name>::new(bytes)),
                    #(#variants)*
                    _ => Err(serde_flow::error::SerdeFlowError::VariantNotFound),
                }
            };
        }

        // Normal - NON ZeroCopy
        let current_dto_name = gen_variant_dto_name(&struct_name);
        let variants: Vec<proc_macro2::TokenStream> = self
            .variants
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|variant| {
                let const_flow_id_name = gen_variant_id_name(&variant);
                let variant_dto_name = gen_variant_dto_name(&variant);
                quote! {
                    #const_flow_id_name => E::deserialize::<#variant_dto_name>(&bytes).map(#variant::from).map(#struct_name::from),
                }
            })
            .collect();
        quote! {
            if bytes.len() < 2 {
                return Err(serde_flow::error::SerdeFlowError::FormatInvalid);
            }
            let flow_id_object = E::deserialize::<serde_flow::flow::FlowId>(&bytes)?;
            match flow_id_object.flow_id {
                #current_variant => E::deserialize::<#current_dto_name>(&bytes).map(#struct_name::from),
                #(#variants)*
                _ => Err(serde_flow::error::SerdeFlowError::VariantNotFound),
            }
        }
    }

    fn component_fs_read(is_bloking: bool) -> proc_macro2::TokenStream {
        if is_bloking {
            return quote! {
                let mut bytes = std::fs::read(path)?;
            };
        }

        #[cfg(feature = "async-std")]
        {
            return quote! {
                let mut bytes = async_std::fs::read(path).await?;
            };
        }

        quote! {
            let mut bytes = tokio::fs::read(path).await?;
        }
    }

    fn component_fs_write(is_bloking: bool) -> proc_macro2::TokenStream {
        if is_bloking {
            return quote! {
                std::fs::write(path, &total_bytes)?;
            };
        }

        #[cfg(feature = "async-std")]
        {
            return quote! {
                async_std::fs::write(path, &total_bytes).await?;
            };
        }
        quote! {
            tokio::fs::write(path, &total_bytes).await?;
        }
    }

    fn component_verify_write(&self, is_bloking: bool) -> proc_macro2::TokenStream {
        let file_read = Self::component_fs_read(is_bloking);
        let file_write = Self::component_fs_write(is_bloking);

        if !self.is_verify_write {
            return quote! {
                #file_write
                Ok(())
            };
        }

        quote! {
            let checksum = serde_flow::encoder::CASTAGNOLI.checksum(&total_bytes);
            let mut attempts = 3;
            while attempts > 0 {
                #file_write
                #file_read
                let written_checksum = serde_flow::encoder::CASTAGNOLI.checksum(&bytes);
                if checksum == written_checksum {
                    return Ok(());
                }
                attempts -= 1;
            }
            Err(serde_flow::error::SerdeFlowError::FailedToWrite)
        }
    }

    pub fn parse(input: DeriveInput) -> syn::parse::Result<Self> {
        let attrs = input.attrs.clone();
        let flow_attr = attrs
            .iter()
            .find(|attr| attr.path().is_ident("flow"))
            .ok_or(syn::parse::Error::new(
                input.span(),
                "flow attribute must be provided",
            ))?;
        let variants_attr = attrs.iter().find(|attr| attr.path().is_ident("variants"));

        // parse structure's fields
        let fields_gen = FieldsGenerator::parse(&input);
        // create the flow generator
        let mut flow_gen = FlowGenerator::new(input.ident.clone(), fields_gen);

        // parse #flow attribute
        flow_gen.parse_flow(flow_attr)?;

        // parse #variants attribute
        if let Some(variants_attr) = variants_attr {
            flow_gen.parse_variants(variants_attr)?;
        }

        Ok(flow_gen)
    }

    fn new(value: Ident, fields_gen: FieldsGenerator) -> Self {
        Self {
            struct_name: value,
            variant: 1,
            is_file: false,
            is_bytes: false,
            is_nonbloking: false,
            is_bloking: false,
            is_zerocopy: false,
            is_verify_write: false,
            variants: None,
            fields_gen,
        }
    }

    fn parse_flow(&mut self, attr: &syn::Attribute) -> syn::parse::Result<()> {
        match &attr.meta {
            syn::Meta::List(meta_list) => meta_list.parse_nested_meta(|meta| {
                if meta.path.is_ident("variant") {
                    let value = meta.value().unwrap(); // this parses the `=`
                    let lit: syn::LitInt = value.parse().unwrap();
                    self.variant = lit.base10_parse::<u16>()?;
                    return Ok(());
                }

                if meta.path.is_ident("bytes") {
                    self.is_bytes = true;
                    return Ok(());
                }
                if meta.path.is_ident("zerocopy") {
                    self.is_zerocopy = true;
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
                            if file_meta.path.is_ident("nonblocking") {
                                self.is_nonbloking = true;
                                return Ok(());
                            }
                            if file_meta.path.is_ident("verify_write") {
                                self.is_verify_write = true;
                                return Ok(());
                            }
                            Err(file_meta.error("unsupported file property"))
                        })?;
                    }
                    // set by default blocking IO
                    if !self.is_bloking && !self.is_nonbloking {
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
            .map(quote::ToTokens::to_token_stream)
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

fn gen_variant_id_name(iden: &Ident) -> Ident {
    Ident::new(
        &format!("FLOW_ID_{}", iden.to_string().to_uppercase()),
        proc_macro2::Span::call_site(),
    )
}

fn gen_variant_dto_name(ident: &Ident) -> Ident {
    Ident::new(&format!("{ident}_FlowDto"), proc_macro2::Span::call_site())
}
