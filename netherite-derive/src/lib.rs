use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Lifetime, LifetimeParam};

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
    let input = parse_macro_input!(tree as DeriveInput);
    let Data::Struct(data) = input.data else { panic!("expected struct") };

    let mut generics = input.generics.clone();
    let de_lifetime: Lifetime = parse_quote!('de);

    let mut lt_param: LifetimeParam = LifetimeParam::new(de_lifetime.clone());
    lt_param
        .bounds
        .extend(generics.lifetimes().map(|lt| &lt.lifetime).cloned());

    generics.params.push(lt_param.into());

    for tp in generics.type_params_mut() {
        tp.bounds.push(parse_quote!(Deserialize<#de_lifetime>))
    }

    let ident = input.ident;
    let (_, struct_generics, _) = input.generics.split_for_impl();
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let fields = data.fields.iter().map(|field| field.ident.as_ref());

    quote!(
        impl #impl_generics netherite::Deserialize<'de> for #ident #struct_generics #where_clause {
            fn deserialize(mut buffer: &mut netherite::encoding::BorrowedBuffer<'de>)
            -> std::result::Result<Self, netherite::DeError> {
                Ok(Self {
                    #(#fields: Deserialize::<'de>::deserialize(&mut buffer)?),*
                })
            }
        }
    )
    .into()
}
