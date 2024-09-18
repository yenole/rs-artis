use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Artis, attributes(artis))]
pub fn device_artis(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    if let syn::Data::Struct(s) = input.data {
        s.fields.iter().for_each(|f| {
            println!("field name: {}", f.ident.as_ref().unwrap());
            println!("field type: {:#?}", f.attrs);
            // match &f.ty {
            //     syn::Type::Path(v) => println!("{:#?}", v.path),
            //     _ => {}
            // };
        });
    }
    // match input.data {
    //     syn::Data::Struct(data) => {
    //         match data.fields {
    //             syn::Fields::Named(fields) => {
    //                 // 遍历 named fields，获取每个字段的类型
    //                 for field in fields.named {
    //                     println!("field name: {}, type: {:#?}", field.ty);
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    //     _ => {}
    // }
    quote! {}.into()
}
