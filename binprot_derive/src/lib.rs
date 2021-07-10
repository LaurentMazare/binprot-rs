extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed, GenericParam,
};

// TODO: maybe add also a deriver for BinProtSize?
#[proc_macro_derive(BinProtWrite, attributes(polymorphic))]
pub fn binprot_write_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binprot_write(&ast)
}

fn impl_binprot_write(ast: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(BinProtWrite))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impl_fn = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { self.#name.binprot_write(w)?; }
                });
                quote! {#(#fields)*}
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let num_fields = unnamed.len();
                let fields = (0..num_fields).map(|index| {
                    let index = syn::Index::from(index);
                    quote! { self.#index.binprot_write(w)?; }
                });
                quote! {#(#fields)*}
            }
            syn::Fields::Unit => {
                unimplemented!()
            }
        },
        syn::Data::Enum(DataEnum {
            enum_token,
            variants,
            ..
        }) => {
            if variants.len() > 256 {
                return syn::Error::new_spanned(&enum_token, "enum with to many cases")
                    .to_compile_error()
                    .into();
            }
            let cases = variants.iter().enumerate().map(|(variant_index, variant)| {
                let variant_ident = &variant.ident;
                let (pattern, actions) = match &variant.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let args = named.iter().map(|field| field.ident.as_ref().unwrap());
                        let fields = named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            quote! { #name.binprot_write(w)?; }
                        });
                        (quote! { { #(#args),* } }, quote! { #(#fields)* })
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let num_fields = unnamed.len();
                        let args = (0..num_fields).map(|index| format_ident!("arg{}", index));
                        let write_args = {
                            let args = args.clone();
                            quote! { #(#args.binprot_write(w);)* }
                        };
                        (quote! { (#(#args),*) }, write_args)
                    }
                    syn::Fields::Unit => (quote! { () }, quote! {}),
                };
                quote! {
                    #ident::#variant_ident #pattern => {
                        w.write_all(&[#variant_index as u8])?;
                        #actions
                    }
                }
            });
            quote! {
                match self {
                    #(#cases)*
                };
            }
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(&union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics binprot::BinProtWrite for #ident #ty_generics #where_clause {
            fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
                #impl_fn
                Ok(())
            }
        }
    };

    output.into()
}

#[proc_macro_derive(BinProtRead, attributes(polymorphic))]
pub fn binprot_read_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binprot_read(&ast)
}

fn impl_binprot_read(ast: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(BinProtRead))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (mk_temps, build) = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|field| field.ident.as_ref().unwrap());
                let build = quote! { #ident { #(#fields),* }};
                let mk_fields = named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    quote! { let #name = BinProtRead::binprot_read(r)?; }
                });
                (quote! {#(#mk_fields)*}, build)
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let num_fields = unnamed.len();
                let fields = (0..num_fields).map(|index| format_ident!("__field{}", index));
                let build = quote! { #ident(#(#fields),*)};
                let mk_fields = (0..num_fields).map(|index| {
                    let ident = format_ident!("__field{}", index);
                    quote! { let #ident = BinProtRead::binprot_read(r)?; }
                });
                (quote! {#(#mk_fields)*}, build)
            }
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            unimplemented!()
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(&union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics binprot::BinProtRead for #ident #ty_generics #where_clause {
            fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> std::result::Result<Self, binprot::Error> {
                #mk_temps
                Ok(#build)
            }
        }
    };

    output.into()
}
