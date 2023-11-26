extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;

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
                ret.push_str(&format!("\n    {}", l.value().trim()));
            }
        }
    }

    ret
}

#[proc_macro_derive(EnumDocs)]
/// Returns a const str for the Enum to which it applies.
///
/// # Example:
/// ```
/// #[derive(leftwm_macros::EnumDocs)]
/// enum LeftWm {
///   One,
///   /// Doc comment
///   Two
/// }
///
/// assert_eq!(LeftWm::documentation(), "\nOne\nTwo\n    Doc comment");
/// ```
///
/// The purpose of this macro is for serializing options of the `BaseCommand` for `leftwm-command`
pub fn derive_enum_docs(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    match &input.data {
        // Only if data is an enum, we do parsing
        Data::Enum(data_enum) => {
            // data_enum is of type syn::DataEnum
            // https://doc.servo.org/syn/struct.DataEnum.html

            let mut names = String::new();

            // For each variant, push its name onto `names`
            for variant in &data_enum.variants {
                let doc = parse_enum_doc_comment(&variant.attrs);

                names.push_str(&format!("\n{}{}", variant.ident, doc));
            }

            // The enum's name
            let name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    pub const fn documentation() -> &'static str {
                        #names
                    }
                }
            }
            .into()
        }
        _ => derive_error!("EnumDocs can only be implemented for enums"),
    }
}
