use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, DataStruct, DeriveInput, GenericArgument, PathSegment, Type,
};

fn extrat_colume(v: &PathSegment) -> String {
    if v.arguments.is_empty() {
        return v.ident.to_string();
    }
    let raw = v.ident.to_string();
    match raw.as_str() {
        "Vec" => return "Vec".into(),
        "HashMap" => return "Map".into(),
        _ => {}
    };
    if let syn::PathArguments::AngleBracketed(v) = &v.arguments {
        if let GenericArgument::Type(v) = v.args.first().unwrap() {
            return extrat_type(v);
        }
    }
    "".into()
}

fn extrat_type(t: &syn::Type) -> String {
    if let Type::Path(v) = t {
        return extrat_colume(v.path.segments.first().unwrap());
    }
    return "".into();
}

#[derive(Debug, Clone)]
struct Artis {
    pub name: String,
    pub typ: String,
    pub size: Option<TokenTree>,
    pub nonull: bool,
    pub index: bool,
    pub unique: bool,
    pub primary: bool,
    pub default: String,
    pub comment: String,
    pub increment: bool,
}

impl Default for Artis {
    fn default() -> Self {
        Self {
            name: "".into(),
            typ: "".into(),
            size: Some(Literal::i32_unsuffixed(0).into()),
            nonull: false,
            index: false,
            unique: false,
            primary: false,
            default: "".into(),
            comment: "".into(),
            increment: false,
        }
    }
}

fn extrat_literal(v: Option<TokenTree>) -> String {
    if v.is_none() {
        return "".into();
    }
    v.unwrap().to_string().trim_matches('"').to_string()
}

impl From<TokenStream> for Artis {
    fn from(value: TokenStream) -> Self {
        let mut itr = value.into_iter();
        let mut artis = Artis::default();
        while let Some(v) = itr.next() {
            let raw = v.to_string();
            match raw.as_str() {
                "type" => {
                    itr.next();
                    artis.typ = extrat_literal(itr.next());
                }
                "size" => {
                    itr.next();
                    artis.size = itr.next(); //ktol extrat_literal(itr.next()).parse::<i32>().unwrap();
                }
                "default" => {
                    itr.next();
                    artis.default = extrat_literal(itr.next());
                }
                "comment" => {
                    itr.next();
                    artis.comment = extrat_literal(itr.next());
                }
                "INDEX" => {
                    artis.index = true;
                }
                "UNIQUE" => {
                    artis.unique = true;
                }
                "NOT_NULL" => {
                    artis.nonull = true;
                }
                "PRIMARY" => {
                    artis.primary = true;
                    artis.nonull = true;
                }
                "AUTO_INCREMENT" => {
                    artis.increment = true;
                }
                _ => {}
            }
        }
        artis
    }
}

fn extrat_attrs(list: Vec<Attribute>) -> Option<Artis> {
    for v in list {
        let meta = &v.meta.require_list().unwrap();
        if !meta.path.is_ident("artis") {
            continue;
        }
        return Some(meta.tokens.clone().into());
    }
    None
}

fn extend_feilds(v: &DataStruct) -> Vec<Artis> {
    let mut fields: Vec<Artis> = vec![];
    for field in &v.fields {
        let mut artis = Artis::default();
        if let Some(v) = extrat_attrs(field.attrs.clone()) {
            artis = v;
        }
        artis.name = field.ident.as_ref().unwrap().to_string();
        if artis.typ.is_empty() {
            artis.typ = format!(":{}", extrat_type(&field.ty));
        }
        fields.push(artis);
    }
    fields
}

#[proc_macro_derive(Artis, attributes(artis))]
pub fn device_artis(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;
    let table = format!("{}s", name.to_string().to_lowercase());
    let mut inx_quote: Vec<TokenStream> = vec![];
    let mut com_quote: Vec<TokenStream> = vec![];
    let mut primary = String::new();
    if let syn::Data::Struct(s) = input.data {
        let fields = extend_feilds(&s);
        for field in fields {
            let name = &field.name;
            let colume = field.typ;
            let size = field.size;
            let nullable = !field.nonull;
            let default = field.default;
            let comment = field.comment;
            let increment = field.increment;
            let quote = quote! {artis::migrator::ColumeMeta {
                name:#name.into(),
                colume: #colume.into(),
                size: #size,
                nullable: #nullable,
                default: #default.into(),
                comment: #comment.into(),
                increment:#increment
            }};
            com_quote.push(quote.into());

            if field.primary {
                primary = field.name;
                continue;
            }
            if field.unique {
                inx_quote.push(quote! {
                    artis::migrator::IndexMeta::Unique(#name.into())
                });
                continue;
            }
            if field.index {
                inx_quote.push(quote! {
                    artis::migrator::IndexMeta::Index(#name.into())
                });
            }
        }
    }
    quote! {
        impl artis::migrator::ArtisMigrator for #name {
            fn migrator() -> artis::migrator::TableMeta {
                artis::migrator::TableMeta {
                    name: #table.into(),
                    primary: #primary.into(),
                    columes: vec![#(#com_quote,)*],
                    indexs: vec![#(#inx_quote,)*]
                }
            }
        }
    }
    .into()
}
