use quote::quote;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, DeriveInput, GenericArgument, PathArguments,
    Type,
};

#[proc_macro_derive(Artis, attributes(artis))]
pub fn device_artis(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    if let syn::Data::Struct(s) = input.data {
        s.fields.iter().for_each(|f| {
            println!("field name: {}", f.ident.as_ref().unwrap());
            // println!("field type: {:#?}", f.ty);
            f.attrs.iter().for_each(|a| {
                if let Ok(meta) = a.meta.require_list() {
                    println!("attr: {:#?}", meta.path.get_ident().unwrap().to_string());
                    println!("tokens: {:#?}", meta.tokens);
                }
                // let meta = a.meta.require_list();
                // println!(
                //     "attr: {:#?}",
                //     a.meta.path().get_ident().unwrap().to_string()
                // );
                // println!("attr: {:#?}", a.meta.require_list());
            });
            // 获取类型
            // if let syn::Type::Path(v) = &f.ty {
            //     let first = v.path.segments.first().unwrap();
            //     println!("first type:{:#?}", first.ident);
            //     if let PathArguments::AngleBracketed(v) = &first.arguments {
            //         let secen = v.args.first().unwrap().clone();
            //         if let GenericArgument::Type(v) = secen {
            //             if let Type::Path(v) = v {
            //                 println!("{:#?}", v.path.get_ident());
            //             }
            //         }
            //     }
            // }
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
