extern crate proc_macro;

use proc_macro::{TokenStream};
use proc_macro2::TokenStream as TS;

use quote::{format_ident, quote};
use syn::{Data, Fields, Field, Type, Ident, PathArguments, GenericArgument};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;
    let buildername = format_ident!("{}Builder", name);
    let fields = if let Data::Struct(fields) = input.data {
        fields
    } else {
        unimplemented!();
    };
    let fields = if let Fields::Named(fields) = fields.fields {
        fields
    } else {
        unimplemented!();
    };

    let (names, t): (Vec<Ident>, Vec<(TS, Type)>) = fields.named.into_iter().map(|field: Field| {
        let inner_ty = wrapped_type(&field.ty);
        let name = field.ident.unwrap();
        let (setters, ty) = match inner_ty {
            Some(ty) => {
                (quote!(let #name = self.#name.clone();), ty)
            },
            None => {
                (quote!(
                    let #name = if let Some(#name) = self.#name.clone() {
                        #name
                    } else {
                        return Err(String::from("#name"));
                    };
                ),
                field.ty)
            }
        };
        (name, (setters, ty))
    }).unzip();

    let (setters, types): (Vec<TS>, Vec<Type>) = t.into_iter().unzip();

    let tokens = quote! (
        impl #name {
            pub fn builder() -> #buildername {
                <#buildername>::default()
            }
        }

        #[derive(Debug, Default)]
        pub struct #buildername {
            #(#names: Option<#types>,)*
        }

        impl #buildername {
            #(pub fn #names(&mut self, #names: #types) -> &mut #buildername {
                self.#names = Some(#names);
                self
            })*

            pub fn build(&mut self) -> Result<#name, String> {
                #(#setters)*

                Ok(#name {
                    #(#names,)*
                })
            }
        }
    );

    TokenStream::from(tokens)
}

fn wrapped_type(ty: &Type) -> Option<Type> {
    if let Type::Path(ref ty_path) = ty {
        if ty_path.path.segments.len() != 1 {
            return None;
        }

        let seg = ty_path.path.segments.first().unwrap();
        if seg.ident != "Option" {
            return None;
        }

        if let PathArguments::AngleBracketed(ref gargs) = seg.arguments {
            if let Some(GenericArgument::Type(path)) = gargs.args.first() {
                return Some(path.clone());
            }
        }
    }

    None
}
