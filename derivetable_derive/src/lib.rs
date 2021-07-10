extern crate proc_macro;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;


struct Field<'a> {
    name: &'a syn::Ident,
    inner_type: &'a syn::Type,
}

fn is_index(f: &syn::Field, name: &'static str) -> bool {
    f.attrs
        .iter()
        .find(|attr| attr.path.is_ident(name))
        .is_some()
}

fn get_indexes(data: &syn::DataStruct) -> (Vec<Field>, Vec<Field>, Vec<Field>) {
    let mut indexes = vec![];
    let mut uniques = vec![];
    let mut hindexes = vec![];

    match data.fields {
        syn::Fields::Named(ref named_fields) => {
            for field in &named_fields.named {
                if is_index(field, "index") {
                    indexes.push(Field { name: field.ident.as_ref().unwrap(), inner_type: &field.ty });
                } else if is_index(field, "unique") {
                    uniques.push(Field { name: field.ident.as_ref().unwrap(), inner_type: &field.ty });
                } else if is_index(field, "hindex") {
                    hindexes.push(Field { name: field.ident.as_ref().unwrap(), inner_type: &field.ty });
                }
            }
        },
        _ => {
            panic!("Expected a struct with named fields");
        },
    };

    (indexes, hindexes, uniques)
}

fn emit_idx_init(field: &Field, unique: bool) -> proc_macro2::TokenStream {
    let name = format_ident!("{}idx_{}", if unique { "u" } else { "" }, field.name);
    quote! { #name: Default::default() }
}

fn emit_idx_decl(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let inner_type = field.inner_type;
    quote! { #name: std::collections::BTreeMap<#inner_type, std::collections::BTreeSet<usize>> }
}

fn emit_hidx_decl(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let inner_type = field.inner_type;
    quote! { #name: std::collections::HashMap<#inner_type, std::collections::HashSet<usize>> }
}

fn emit_uidx_decl(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("uidx_{}", field.name);
    let inner_type = field.inner_type;
    quote! { #name: std::collections::BTreeMap<#inner_type, usize> }
}

fn emit_idx_insert(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let fieldname = field.name;
    quote! { 
        let field_c = row.#fieldname.clone();
        let ename = self.#name.entry(field_c)
            .or_insert_with(|| Default::default());
        ename.insert(id);
    }
}

fn emit_unique_check(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("uidx_{}", field.name);
    let fieldname = field.name;
    quote! { 
        match self.#name.get(&row.#fieldname) {
            Some(idx) => return Err(*idx),
            _ => (),
        }
    }
}

fn emit_unique_insert(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("uidx_{}", field.name);
    let fieldname = field.name;
    quote! {
        let field_c = row.#fieldname.clone();
        self.#name.insert(field_c, id);
    }
}

fn emit_remove_index(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let fieldname = field.name;
    quote! {
        let mut clean = false;
        match self.#name.get_mut(&row.#fieldname) {
            Some(set) => {
                set.remove(&id);
                if set.is_empty() {
                    clean = true;
                }
            },
            None => {
            },
        }

        if clean {
            self.#name.remove(&row.#fieldname);
        }
    }
}
   
fn emit_remove_unique(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!("uidx_{}", field.name);
    let fieldname = field.name;
    quote! { self.#name.remove(&row.#fieldname); }
}

fn emit_queries_by_unique(field: &Field, rowtype: &syn::Ident, pub_d: &Option<syn::Ident>) -> proc_macro2::TokenStream {
    let name = format_ident!("uidx_{}", field.name);
    let fieldname = field.name;
    let fn_name = format_ident!("get_by_{}", fieldname);
    let ty = field.inner_type;

    quote! {
        #pub_d fn #fn_name <'a>(&'a self, #name: &#ty) -> Option<&'a #rowtype> {
            self.#name.get(#name)
                .map(|iid| &self.data[*iid])
        }
    }
}

fn emit_queries_by_index(field: &Field, rowtype: &syn::Ident, pub_d: &Option<syn::Ident>) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let fieldname = field.name;
    let get_fn_name = format_ident!("get_by_{}", fieldname);
    let range_fn_name = format_ident!("range_by_{}", fieldname);
    let ty = field.inner_type;

    quote! {
        #pub_d fn #get_fn_name <'a>(&'a self, #name: &#ty)
            -> impl DoubleEndedIterator<Item = (usize, &'a #rowtype)> + 'a 
        {
            let idxs = self.#name.get(#name)
                .into_iter()
                .map(|idx_set| idx_set.iter())
                .flatten()
                .cloned();

            derivetable::IndexDoubleEndedIterator { data: &self.data, idxs }
        }

        #pub_d fn #range_fn_name <'a, R>(&'a self, range: R) 
            -> impl DoubleEndedIterator<Item = (usize, &'a #rowtype)> + 'a
            where
                R: std::ops::RangeBounds<#ty>
        {
            let idxs = self.#name.range(range)
                .map(|(_, idx_set)| idx_set.iter())
                .flatten()
                .cloned();

            derivetable::IndexDoubleEndedIterator { data: &self.data, idxs }
        }
    }
}

fn emit_queries_by_hindex(field: &Field, rowtype: &syn::Ident, pub_d: &Option<syn::Ident>) -> proc_macro2::TokenStream {
    let name = format_ident!("idx_{}", field.name);
    let fieldname = field.name;
    let get_fn_name = format_ident!("get_by_{}", fieldname);
    let ty = field.inner_type;

    quote! {
        #pub_d fn #get_fn_name <'a>(&'a self, #name: &#ty)
            -> impl Iterator<Item = (usize, &'a #rowtype)> + 'a 
        {
            let idxs = self.#name.get(#name)
                .into_iter()
                .map(|idx_set| idx_set.iter())
                .flatten()
                .cloned();

            derivetable::IndexIterator { data: &self.data, idxs }
        }
    }
}

fn get_derives<'a>(attrs: &'a [syn::Attribute]) -> Vec<syn::Ident> {
    let mut res = vec![];
    attrs.into_iter()
        .find(|attr| attr.path.is_ident("derivetable"))
        .map(|attr| attr.tokens
            .clone()
            .into_iter()
            .take(1)
            .for_each(|t| {
                if let proc_macro2::TokenTree::Group(group) = t {
                    for item in group.stream().into_iter() {
                        if let proc_macro2::TokenTree::Ident(ref ident) = item {
                            res.push(format_ident!("{}", ident.to_string()));
                        }
                    }
                }
            })
        );

    res
}

#[proc_macro_derive(Table, attributes(index, hindex, unique, derivetable))]
pub fn derivetable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ds = match input.data {
        syn::Data::Struct(ref datastruct) => datastruct,
        _ => panic!("Expected struct"),
    };

    let table_derives = get_derives(&input.attrs);
    
    let (indexes, hindexes, uniques) = get_indexes(&ds);
    let rowtype = input.ident;

    let pub_d = match input.vis {
        syn::Visibility::Public(_) => Some(format_ident!("pub")),
        syn::Visibility::Crate(_) => Some(format_ident!("pub(crate)")),
        _ => None,
    };

    let idx_fields_decls = indexes.iter().map(emit_idx_decl);
    let idx_fields_inits = indexes.iter().map(|f| emit_idx_init(f, false));
    let hidx_fields_decls = hindexes.iter().map(emit_hidx_decl);
    let hidx_fields_inits = hindexes.iter().map(|f| emit_idx_init(f, false));
    let idx_uniques_decls = uniques.iter().map(emit_uidx_decl);
    let idx_uniques_inits = uniques.iter().map(|f| emit_idx_init(f, true));
    let insert_indexes = indexes.iter().map(emit_idx_insert);
    let insert_hindexes = hindexes.iter().map(emit_idx_insert);
    let check_uniques = uniques.iter().map(emit_unique_check);
    let insert_uniques = uniques.iter().map(emit_unique_insert);
    let remove_indexes = indexes.iter().map(emit_remove_index);
    let remove_indexes2 = remove_indexes.clone();
    let remove_hindexes = hindexes.iter().map(emit_remove_index);
    let remove_hindexes2 = remove_indexes.clone();
    let remove_uniques = uniques.iter().map(emit_remove_unique);
    let remove_uniques2 = remove_uniques.clone();
    let queries_by_index = indexes.iter().map(|f| emit_queries_by_index(f, &rowtype, &pub_d));
    let queries_by_hindex = hindexes.iter().map(|f| emit_queries_by_hindex(f, &rowtype, &pub_d));
    let queries_by_unique = uniques.iter().map(|f| emit_queries_by_unique(f, &rowtype, &pub_d));
    let insert_indexes2 = insert_indexes.clone();
    let insert_hindexes2 = insert_hindexes.clone();
    let insert_uniques2 = insert_uniques.clone();

    let table_type = format_ident!("{}Table", rowtype);

    let expanded = quote! {
        #[derive(#(#table_derives,)*)]
        #pub_d struct #table_type {
            data: Vec<#rowtype>,
            #(#idx_fields_decls,)*
            #(#hidx_fields_decls,)*
            #(#idx_uniques_decls,)*
        }

        impl #table_type {
            #pub_d fn new() -> #table_type {
                #table_type {
                    data: Default::default(),
                    #(#idx_fields_inits ,)*
                    #(#hidx_fields_inits ,)*
                    #(#idx_uniques_inits ,)*
                }
            }

            #pub_d fn iter(&self) -> impl DoubleEndedIterator<Item=&#rowtype> {
                self.data.iter()
            }

            #pub_d fn insert(&mut self, row: #rowtype) -> std::result::Result<usize, usize> {
                let id = self.data.len();
                self.insert_inner(id, row)
            }

            fn insert_inner(&mut self, id: usize, row: #rowtype) -> std::result::Result<usize, usize> {
                #(#check_uniques)*
                #(#insert_indexes)*
                #(#insert_hindexes)*
                #(#insert_uniques)*

                self.data.push(row);

                Ok(id)
            }

            #pub_d fn remove(&mut self, id: usize) -> Option<#rowtype> {
                if id < self.data.len() {
                    let row = &self.data[id];
                    // remove indexes of target id
                    #(#remove_indexes)*
                    #(#remove_hindexes)*
                    #(#remove_uniques)*
                    let oldid = id;

                    // remove indexes for the one to be swapped with
                    if self.data.len() > 0 {
                        let id = self.data.len() - 1;
                        let row = &self.data[id];
                        #(#remove_indexes2)*
                        #(#remove_hindexes2)*
                        #(#remove_uniques2)*
                    }

                    let id = oldid;
                    let out_row = self.data.swap_remove(id);
                    let row = &self.data[id];
                    // update indexes
                    #(#insert_indexes2)*
                    #(#insert_hindexes2)*
                    #(#insert_uniques2)*

                    Some(out_row)
                } else {
                    None
                }
            }

            fn get(&self, id: usize) -> Option<&#rowtype> {
                if id < self.data.len() {
                    Some(&self.data[id])
                } else {
                    None
                }
            }

            /*
            #pub_d fn update<F: Fn(&mut #rowtype)>(&mut self, id: &usize, fun: F) {
                #(#remove_indexes)*
                #(#remove_hindexes)*
                #(#remove_uniques)*
                
                if id < self.data.len() {
                    fun(&mut sef.data[id]);

                    //#(#check_uniques)*
                    #(#insert_indexes)*
                    #(#insert_hindexes)*
                    #(#insert_uniques)*
                }
            }*/

            #(#queries_by_index)*
            #(#queries_by_hindex)*
            #(#queries_by_unique)*
        }
    };

    // to debug the proc macro uncomment this
    //println!("{}", &expanded.to_string());

    TokenStream::from(expanded)
}
