// TODO: maybe add also a deriver for BinProtSize?
use ::proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed, GenericParam,
};

fn has_polymorphic_variant_attr(ast: &DeriveInput) -> bool {
    let DeriveInput { attrs, .. } = ast;
    attrs.iter().any(|attr| {
        attr.path.segments.len() == 1 && attr.path.segments[0].ident == "polymorphic_variant"
    })
}

// The hash function used to get the identifier for polymorphic variants in OCaml
fn hash_variant(str: &str) -> i32 {
    let mut accu = std::num::Wrapping(0i64);
    for &v in str.as_bytes().iter() {
        accu = std::num::Wrapping(223) * accu + std::num::Wrapping(v as i64)
    }
    accu &= std::num::Wrapping((1 << 31) - 1);
    let accu = accu.0;
    if accu > 0x3FFFFFFF {
        (accu - (1 << 31)) as i32
    } else {
        accu as i32
    }
}

// https://github.com/janestreet/bin_prot/blob/5915cde59105f398b53f682c5f4dad29e272f696/src/write.ml#L387-L393
fn variant_int(str: &str) -> i32 {
    let v = hash_variant(str);
    (v << 1) | 1
}

#[proc_macro_derive(BinProtWrite, attributes(polymorphic_variant))]
pub fn binprot_write_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binprot_write(&ast)
}

fn impl_binprot_write(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(BinProtWrite))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let has_polymorphic_variant_attr = has_polymorphic_variant_attr(ast);

    let impl_fn = match data {
        syn::Data::Struct(s) => {
            if has_polymorphic_variant_attr {
                panic!("polymorphic_variant is only allowed on enum")
            }
            match &s.fields {
                syn::Fields::Named(FieldsNamed { named, .. }) => {
                    let fields = named.iter().map(|field| {
                        let name = field.ident.as_ref().unwrap();
                        quote! { self.#name.binprot_write(__binprot_w)?; }
                    });
                    quote! {#(#fields)*}
                }
                syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    let num_fields = unnamed.len();
                    let fields = (0..num_fields).map(|index| {
                        let index = syn::Index::from(index);
                        quote! { self.#index.binprot_write(__binprot_w)?; }
                    });
                    quote! {#(#fields)*}
                }
                syn::Fields::Unit => {
                    unimplemented!()
                }
            }
        }
        syn::Data::Enum(DataEnum { enum_token, variants, .. }) => {
            if variants.len() > 256 {
                return syn::Error::new_spanned(enum_token, "enum with to many cases")
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
                            quote! { #name.binprot_write(__binprot_w)?; }
                        });
                        (quote! { { #(#args),* } }, quote! { #(#fields)* })
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let num_fields = unnamed.len();
                        let args = (0..num_fields).map(|index| format_ident!("arg{}", index));
                        let write_args = {
                            let args = args.clone();
                            quote! { #(#args.binprot_write(__binprot_w)?;)* }
                        };
                        (quote! { (#(#args),*) }, write_args)
                    }
                    syn::Fields::Unit => (quote! {}, quote! {}),
                };
                let variant_index = if !has_polymorphic_variant_attr {
                    let variant_index = variant_index as u8;
                    quote! { [#variant_index] }
                } else {
                    let variant_index: i32 = variant_int(&variant_ident.to_string());
                    quote! { (#variant_index).to_le_bytes() }
                };
                quote! {
                    #ident::#variant_ident #pattern => {
                        __binprot_w.write_all(&#variant_index)?;
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
            return syn::Error::new_spanned(union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics binprot::BinProtWrite for #ident #ty_generics #where_clause {
            fn binprot_write<__BinProtW: std::io::Write>(&self, __binprot_w: &mut __BinProtW) -> std::io::Result<()> {
                #impl_fn
                Ok(())
            }
        }
    };

    output.into()
}

#[proc_macro_derive(BinProtRead, attributes(polymorphic_variant))]
pub fn binprot_read_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binprot_read(&ast)
}

fn impl_binprot_read(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(BinProtRead))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let has_polymorphic_variant_attr = has_polymorphic_variant_attr(ast);

    let read_fn = match data {
        syn::Data::Struct(s) => {
            if has_polymorphic_variant_attr {
                panic!("polymorphic_variant is only allowed on enum")
            }

            match &s.fields {
                syn::Fields::Named(FieldsNamed { named, .. }) => {
                    let fields = named.iter().map(|field| field.ident.as_ref().unwrap());
                    let mk_fields = named.iter().map(|field| {
                        let name = field.ident.as_ref().unwrap();
                        quote! { let #name = binprot::BinProtRead::binprot_read(__binprot_r)?; }
                    });
                    quote! {
                        #(#mk_fields)*
                        Ok(#ident { #(#fields),* })
                    }
                }
                syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    let num_fields = unnamed.len();
                    let fields = (0..num_fields).map(|index| format_ident!("__field{}", index));
                    let mk_fields = (0..num_fields).map(|index| {
                        let ident = format_ident!("__field{}", index);
                        quote! { let #ident = binprot::BinProtRead::binprot_read(__binprot_r)?; }
                    });
                    quote! {
                        #(#mk_fields)*
                        Ok(#ident(#(#fields),*))
                    }
                }
                syn::Fields::Unit => unimplemented!(),
            }
        }
        syn::Data::Enum(DataEnum { enum_token, variants, .. }) => {
            if variants.len() > 256 {
                return syn::Error::new_spanned(enum_token, "enum with to many cases")
                    .to_compile_error()
                    .into();
            }
            let cases = variants.iter().enumerate().map(|(variant_index, variant)| {
                let variant_ident = &variant.ident;
                let variant_index = if !has_polymorphic_variant_attr {
                    let variant_index = variant_index as u8;
                    quote! { #variant_index }
                } else {
                    let variant_index: i32 = variant_int(&variant_ident.to_string());
                    quote! { #variant_index }
                };
                let (mk_fields, fields) = match &variant.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let fields = named.iter().map(|field| field.ident.as_ref().unwrap());
                        let mk_fields = named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            quote! { let #name = binprot::BinProtRead::binprot_read(__binprot_r)?; }
                        });
                        (quote! { #(#mk_fields)* }, quote! { { #(#fields),* } })
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let num_fields = unnamed.len();
                        let fields = (0..num_fields).map(|index| format_ident!("__field{}", index));
                        let mk_fields = (0..num_fields).map(|index| {
                            let ident = format_ident!("__field{}", index);
                            quote! { let #ident = binprot::BinProtRead::binprot_read(__binprot_r)?; }
                        });
                        (quote! { #(#mk_fields)* }, quote! { (#(#fields),*) })
                    }
                    syn::Fields::Unit => (quote! {}, quote! {}),
                };
                quote! {
                    #variant_index => {
                        #mk_fields
                        Ok(#ident::#variant_ident #fields)
                    }
                }
            });
            if !has_polymorphic_variant_attr {
                quote! {
                    let variant_index = binprot::byteorder::ReadBytesExt::read_u8(__binprot_r)?;
                    match variant_index {
                        #(#cases)*
                        index => Err(binprot::Error::UnexpectedVariantIndex { index, ident: stringify!(#ident) } ),
                    }
                }
            } else {
                quote! {
                    let variant_index = binprot::byteorder::ReadBytesExt::read_i32::<binprot::byteorder::LittleEndian>(__binprot_r)?;
                    match variant_index {
                        #(#cases)*
                        index => Err(binprot::Error::UnexpectedPolymorphicVariantIndex { index, ident: stringify!(#ident) } ),
                    }
                }
            }
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics binprot::BinProtRead for #ident #ty_generics #where_clause {
            fn binprot_read<__BinProtR: std::io::Read + ?Sized>(__binprot_r: &mut __BinProtR) -> std::result::Result<Self, binprot::Error> {
                #read_fn
            }
        }
    };

    output.into()
}

#[proc_macro_derive(BinProtShape, attributes(polymorphic_variant))]
pub fn binprot_shape_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binprot_shape(&ast)
}

fn impl_binprot_shape(ast: &DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(BinProtShape))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let has_polymorphic_variant_attr = has_polymorphic_variant_attr(ast);

    let impl_fn = match data {
        syn::Data::Struct(s) => {
            if has_polymorphic_variant_attr {
                panic!("polymorphic_variant is only allowed on enum")
            }
            match &s.fields {
                syn::Fields::Named(FieldsNamed { named, .. }) => {
                    let fields = named.iter().map(|field| {
                        let name = field.ident.as_ref().unwrap();
                        let ty = &field.ty;
                        quote! { (stringify!(#name), <#ty>::binprot_shape_loop(_c)) }
                    });
                    quote! {binprot::Shape::Record(vec![#(#fields),*])}
                }
                syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    let fields = unnamed.iter().map(|field| {
                        let ty = &field.ty;
                        quote! {<#ty>::binprot_shape_loop(_c) }
                    });
                    quote! {binprot::Shape::Tuple(vec![#(#fields,)*])}
                }
                syn::Fields::Unit => {
                    unimplemented!()
                }
            }
        }
        syn::Data::Enum(DataEnum { variants, .. }) => {
            if has_polymorphic_variant_attr {
                let cases = variants.iter().map(|variant| {
                    let args = match &variant.fields {
                        syn::Fields::Named(FieldsNamed { named, .. }) => {
                            let fields = named.iter().map(|field| {
                                let name = field.ident.as_ref().unwrap();
                                let ty = &field.ty;
                                quote! { (stringify!(#name), <#ty>::binprot_shape_loop(_c)) }
                            });
                            quote! {Some(binprot::Shape::Record(vec![#(#fields),*]))}
                        }
                        syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            let tuple = unnamed
                                .iter()
                                .map(|field| {
                                    let ty = &field.ty;
                                    quote! {<#ty>::binprot_shape_loop(_c) }
                                })
                                .collect::<Vec<_>>();
                            if tuple.len() == 1 {
                                let tuple = &tuple[0];
                                quote! {Some(#tuple)}
                            } else {
                                quote! {Some(binprot::Shape::Tuple(vec![#(#tuple),*]))}
                            }
                        }
                        syn::Fields::Unit => quote! {None},
                    };
                    let name = &variant.ident;
                    quote! {(stringify!(#name), #args)}
                });
                quote! {
                    binprot::Shape::PolyVariant(vec![#(#cases,)*].into_iter().collect())
                }
            } else {
                let cases = variants.iter().map(|variant| {
                    let args = match &variant.fields {
                        syn::Fields::Named(FieldsNamed { named, .. }) => {
                            let fields = named.iter().map(|field| {
                                let name = field.ident.as_ref().unwrap();
                                let ty = &field.ty;
                                quote! { (stringify!(#name), <#ty>::binprot_shape_loop(_c)) }
                            });
                            vec![quote! {binprot::Shape::Record(vec![#(#fields),*])}]
                        }
                        syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed
                            .iter()
                            .map(|field| {
                                let ty = &field.ty;
                                quote! {<#ty>::binprot_shape_loop(_c) }
                            })
                            .collect::<Vec<_>>(),
                        syn::Fields::Unit => vec![],
                    };
                    let name = &variant.ident;
                    quote! {(stringify!(#name), vec![#(#args,)*])}
                });
                quote! {
                    binprot::Shape::Variant(vec![#(#cases,)*])
                }
            }
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics binprot::BinProtShape for #ident #ty_generics #where_clause {
            fn binprot_shape_impl(_c: &mut binprot::ShapeContext) -> binprot::Shape {
                #impl_fn
            }
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_variant() {
        assert_eq!(hash_variant(""), 0);
        assert_eq!(hash_variant("foo"), 5097222);
        assert_eq!(hash_variant("FooBar"), 805748365);
        assert_eq!(hash_variant("FooBarBazAndEvenMoreAlternatives"), 74946334);
    }
}
