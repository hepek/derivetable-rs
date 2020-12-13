extern crate proc_macro;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

struct Field<'a> {
    name: syn::Ident,
    inner_type: &'a syn::Type,
}

fn is_index(f: &syn::Field) -> bool {
    f.attrs
        .iter()
        .find(|attr| attr.path.is_ident("index"))
        .is_some()
}

fn is_unique(f: &syn::Field) -> bool {
    f.attrs
        .iter()
        .find(|attr| attr.path.is_ident("unique"))
        .is_some()
}

fn get_indexes(data: &syn::DataStruct) -> (Vec<Field>, Vec<Field>) {
    let mut indexes = vec![];
    let mut uniques = vec![];

    match data.fields {
        syn::Fields::Named(ref named_fields) => {
            for field in &named_fields.named {
                if is_index(field) {
                    indexes.push(Field { name: format_ident!("idx_{}", field.ident.as_ref().unwrap()), inner_type: &field.ty });
                } else if is_unique(field) {
                    uniques.push(Field { name: format_ident!("uidx_{}", field.ident.as_ref().unwrap()), inner_type: &field.ty });
                }
            }
        },
        _ => {
            panic!("Expected a struct with named fields");
        },
    };

    (indexes, uniques)
}

#[proc_macro_derive(Table, attributes(index, unique))]
pub fn derivetable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ds = match input.data {
        syn::Data::Struct(ref datastruct) => datastruct,
        _ => panic!("Expected struct"),
    };
    
    let (indexes, uniques) = get_indexes(&ds);

    let idx_fields = indexes.iter().map(|Field { name, inner_type }| {
            quote! { #name: std::collections::BTreeMap<#inner_type, std::collections::BTreeSet<usize>> }
        });

    let idx_fields_inits = indexes.iter().map(|Field { name, .. }| {
            quote! { #name: Default::default() }
        });

    let idx_uniques = uniques.iter().map(|Field { name, inner_type }| {
            quote! { #name: std::collections::BTreeMap<#inner_type, usize> }
        });

    let idx_uniques_inits = uniques.iter().map(|Field { name, .. }| {
            quote! { #name: Default::default() }
        });

    let rowtype = input.ident;
    let table_type = format_ident!("{}Table", rowtype);

    let expanded = quote! {
        struct #table_type {
            next_id: usize,
            data: std::collections::BTreeMap<usize, #rowtype>,
            #(#idx_fields,)*
            #(#idx_uniques,)*
        }

        impl #table_type {
            pub fn new() -> #table_type {
                #table_type {
                    next_id: 0usize,
                    data: Default::default(),
                    #(#idx_fields_inits ,)*
                    #(#idx_uniques_inits ,)*
                }
            }

            pub fn iter(&self) -> impl DoubleEndedIterator<Item=(&usize, &#rowtype)> {
                self.data.iter()
            }

            /*
            pub fn insert(&mut self, row: #rowtype) -> (usize, Vec<#rowtype>) {
                let id = self.next_id;
                self.next_id += 1;

                self.data.insert_inner(id, row);
            }

            fn insert_iner(&mut self, id: usize, row: #rowtype) -> (usize, Vec<#rowtype>) {
                let clone = row.clone();
                self.data.insert(id, row);

                let ename = self.idx_name.entry(clone.name)
                    .or_insert_with(|| Default::default());
                ename.insert(id);
                let edate = self.idx_date.entry(clone.date)
                    .or_insert_with(|| Default::default());
                edate.insert(id);

                let mut to_remove = vec![];

                if let Some(id) = self.idx_id.get(&clone.id) {
                    to_remove.push(id);
                }

                let mut removed = vec![];

                if let Some(old_id) = self.idx_id.insert(clone.id, id) {
                    removed.push(self.remove(&old_id).unwrap());
                }

                (id, removed)
            }
            */
        }
    };

    println!("{:#?}", expanded);
    TokenStream::from(expanded)
}
