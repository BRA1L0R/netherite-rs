use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput};

#[proc_macro_derive(Serialize)]
pub fn serialize(tree: TokenStream) -> TokenStream {
    let mut serialize_struct = parse_macro_input!(tree as DeriveInput);
    let Data::Struct(data) = serialize_struct.data else { panic!("expected struct") };

    let serialize = data.fields.iter().map(|field| field.ident.as_ref());
    let size = serialize.clone();

    for type_param in serialize_struct.generics.type_params_mut() {
        type_param.bounds.push(parse_quote!(Serialize))
    }

    let ident = serialize_struct.ident;
    let (impl_generics, ty_generics, where_generics) = serialize_struct.generics.split_for_impl();

    quote!(
        impl #impl_generics netherite::Serialize for #ident #ty_generics #where_generics {
            fn serialize(&self, mut buf: impl bytes::BufMut) {
                #(self.#serialize.serialize(&mut buf);)*
            }

            fn size(&self) -> usize {
                0 #( + self.#size.size())*
            }
        }
    )
    .into()
}

#[proc_macro_derive(Deserialize)]
pub fn deserialize(tree: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(tree as DeriveInput);
    let Data::Struct(data) = input.data else { panic!("expected struct") };

    for tp in input.generics.type_params_mut() {
        tp.bounds.push(parse_quote!(Deserialize))
    }

    let ident = input.ident;
    let (impl_generics, struct_generics, where_clause) = input.generics.split_for_impl();
    let fields = data.fields.iter().map(|field| field.ident.as_ref());

    quote!(
        impl #impl_generics netherite::Deserialize for #ident #struct_generics #where_clause {
            fn deserialize(mut buffer: impl bytes::Buf)
            -> std::result::Result<Self, netherite::DeError> {
                Ok(Self {
                    #(#fields: Deserialize::deserialize(&mut buffer)?),*
                })
            }
        }
    )
    .into()
}
