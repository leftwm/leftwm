extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

fn parse_enum_doc_comment(attrs: &[syn::Attribute]) -> String {
    let mut ret = String::new();
    for attr in attrs {
        let meta = &attr.meta;
        if let syn::Meta::NameValue(meta) = meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(l),
                ..
            }) = &meta.value
            {
                ret.push_str(&format!("\n          {}", l.value().trim()));
            }
        }
    }

    ret
}

#[proc_macro_derive(VariantNames)]
/// Returns a Vec<String> for the Enum to which it applies.
///
/// # Example:
/// ```
/// #[derive(VariantNames)]
/// enum LeftWm {
///   One,
///   Two
/// }
///
/// assert_eq!(LeftWm::variant_names(), vec!["        One", "        Two"]);
/// ```
///
/// The purpose of this macro is for serializing options of the `BaseCommand` for `leftwm-command`
pub fn derive_variant_names(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    // Get the enum name for use later
    let name = &input.ident;
    // Get the variants
    let data = &input.data;

    let mut variant_checker_functions;

    match data {
        // Only if data is an enum, we do parsing
        Data::Enum(data_enum) => {
            // data_enum is of type syn::DataEnum
            // https://doc.servo.org/syn/struct.DataEnum.html

            variant_checker_functions = TokenStream2::new();

            // For each variant, push its name onto `names`
            let mut names = Vec::new();
            for variant in &data_enum.variants {
                let doc = parse_enum_doc_comment(&variant.attrs);

                names.push(format!("{} {}", variant.ident, doc));
            }

            // Construct the variant_names function for the Enum using `names`
            variant_checker_functions.extend(quote! {
                pub fn variant_names() -> Vec<String> {
                    return vec![#(#names,)*].into_iter().map(String::from).collect();
                }
            });
        }
        _ => return derive_error!("VariantNames can only be implemented for enums"),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #variant_checker_functions
        }
    };

    TokenStream::from(expanded)
}
