extern crate proc_macro;

use proc_macro::{TokenStream};

use quote::{format_ident, quote};
use syn::{Data, Fields, Field, Type, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;
    let buildername = format_ident!("{}Builder", name);
    let fields = if let Data::Struct(fields) = input.data {
        fields
    } else {
        panic!("Not a struct");
    };
    let fields = if let Fields::Named(fields) = fields.fields {
        fields
    } else {
        panic!("Not named!");
    };

    let (names, types): (Vec<Ident>, Vec<Type>) = fields.named.into_iter().map(|field: Field| {
        (field.ident.unwrap(), field.ty)
    }).unzip();

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
                #(
                    let #names = if let Some(#names) = self.#names.clone() {
                        #names
                    } else {
                        return Err(String::from("#names"));
                    };
                )*
                Ok(#name {
                    #(#names,)*
                })
            }
        }
    );

    TokenStream::from(tokens)
}
