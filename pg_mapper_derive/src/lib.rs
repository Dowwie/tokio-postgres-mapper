extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{
    Data, DataStruct, DeriveInput,  Ident, ImplGenerics, Item, 
    Meta::{List, NameValue}, NestedMeta::Meta, TypeGenerics, WhereClause,
};


#[proc_macro_derive(PostgresMapper, attributes(pg_mapper))]
pub fn postgres_mapper(input: TokenStream) -> TokenStream {
    let mut ast: DeriveInput = syn::parse(input).expect("Couldn't parse item");

    impl_derive(&mut ast)
}

fn impl_derive(ast: &mut DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let table_name = parse_table_attr(&ast);

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let s = match ast.data {
        Data::Struct(ref s) => s,
        _ => panic!("Enums or Unions can not be mapped"),
    };

    let tokio_pg_mapper =
        impl_tokio_pg_mapper(s, name, &table_name, impl_generics, ty_generics, where_clause);

    let tokens = quote! {
        #tokio_pg_mapper
    };

    tokens.into()
}


fn impl_tokio_pg_mapper(
    s: &DataStruct,
    name: &Ident,
    table_name: &str,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> Item {
    let fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let row_expr = format!(r##"{}"##, ident);
        quote! {
            #ident:row.try_get::<&str,#ty>(#row_expr)?
        }
    });
    
    let ref_fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let row_expr = format!(r##"{}"##, ident);
        quote! {
            #ident:row.try_get::<&str,#ty>(&#row_expr)?
        }
    });

    let table_columns = 
        s.fields.iter()
              .map(|field| {
        let ident = field.ident.as_ref().expect("Expected structfield identifier");
        format!(" {0}.{1} ", table_name, ident)
    }).collect::<Vec<String>>().join(", ");

    let columns = 
        s.fields.iter()
              .map(|field| {
        let ident = field.ident.as_ref().expect("Expected structfield identifier");
        format!(" {} ", ident)
    }).collect::<Vec<String>>().join(", ");


    let tokens = quote! {
        impl #impl_generics tokio_pg_mapper::FromTokioPostgresRow for #name #ty_generics #where_clause {
            fn from_row(row: tokio_postgres::row::Row) -> ::std::result::Result<Self, tokio_pg_mapper::Error> {
                Ok(Self {
                    #(#fields),*
                })
            }

            fn from_row_ref(row: &tokio_postgres::row::Row) -> ::std::result::Result<Self, tokio_pg_mapper::Error> {
                Ok(Self {
                    #(#ref_fields),*
                })
            }

            fn from_rows(rows: Vec<&tokio_postgres::row::Row>) -> ::std::result::Result<Vec<Self>, tokio_pg_mapper::Error> {
                rows.iter().map(|row| Self::from_row_ref(row).map_err(|e| e.into())).collect()
            }

            fn sql_table() -> String {
                #table_name.to_string()
            }

            fn sql_table_fields() -> String {
                #table_columns.to_string()
            }
            
            fn sql_fields() -> String {
                #columns.to_string()
            }
        }
    };

    syn::parse_quote!(#tokens)
}


fn get_mapper_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "pg_mapper" {
        match attr.parse_meta() {
            Ok(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                panic!("declare table name: #[pg_mapper(table = \"foo\")]");
            }
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    attr_name: Option<&Ident>,
    lit: &'a syn::Lit,
) -> ::std::result::Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        if let Some(val) = attr_name {
            panic!("expected pg_mapper {:?} attribute to be a string", val);
        } else {
            panic!("expected pg_mapper attribute to be a string");
        }
        #[allow(unreachable_code)]
        Err(())
    }
}

fn parse_table_attr(ast: &DeriveInput) -> String {
    // Parse `#[pg_mapper(table = "foo")]`
    let mut table_name: Option<String> = None;

    for meta_items in ast.attrs.iter().filter_map(get_mapper_meta_items) {
        for meta_item in meta_items {
            match meta_item {
                // Parse `#[pg_mapper(table = "foo")]`
                Meta(NameValue(ref m)) if m.path.is_ident("table") => {
                    if let Ok(s) = get_lit_str(m.path.get_ident(), &m.lit) {
                        table_name = Some(s.value());
                    }
                },
                Meta(_) => {
                    panic!(format!(
                        "unknown pg_mapper container attribute",
                    ))
                },
                _ => {
                    panic!("unexpected literal in pg_mapper container attribute");
                }
            }
        }
    }

    table_name.expect("declare table name: #[pg_mapper(table = \"foo\")]")
}
