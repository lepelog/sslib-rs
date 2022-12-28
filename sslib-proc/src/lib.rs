use proc_macro::{TokenStream};
use proc_macro2::{Span, Ident};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type, token::Struct};

// these types get special handling, they are a Vec3 and
// this function returns their inner type
fn get_special_type_ty(typ: &Type) -> Option<&'static str> {
    match typ {
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.iter().next() {
                if seg.ident == "Vec3f" {
                    return Some("f32");
                } else if seg.ident == "Vec3s" {
                    return Some("u16");
                }
            } 
        },
        _ => {}
    }
    None
}

#[proc_macro_attribute]
pub fn derive_patch_match_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let cloned_input = input.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut patch_struct_inner = quote!();
    let mut patch_func_inner = quote!();
    let mut match_func_inner = quote!();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                for field in named.named.iter() {
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    
                    patch_struct_inner.extend(quote!(
                        #name: Option<#ty>,
                    ));

                    patch_func_inner.extend(quote!(
                        if let Some(val) = &patch.#name {
                            self.#name = *val;
                        }
                    ));

                    match_func_inner.extend(quote!(
                        if let Some(val) = &partial.#name {
                            if self.#name != *val {
                                return false;
                            }
                        }
                    ));
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let patch_name: Ident = syn::parse_str(&format!("{name}Patch")).unwrap();

    let gen = quote!(
        #cloned_input

        pub struct #patch_name {
            #patch_struct_inner
        }

        impl #impl_generics #name #ty_generics #where_clause {
            fn patch(&mut self, patch: &#patch_name) {
                #patch_func_inner
            }

            fn matches(&self, partial: &#patch_name) -> bool {
                #match_func_inner
                true
            }
        }
    );

    gen.into()
}

#[proc_macro_derive(SetByName)]
pub fn set_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut gen_inner = quote!();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                for field in named.named.iter() {
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    if let Some(special_type) = get_special_type_ty(ty).map(|c| Ident::new(c, Span::call_site())) {
                        // use a special case for Vec3{f,s} types, so they can be written as `posx` etc.
                        for suffix in ["x", "y", "z"].map(|c| Ident::new(c, Span::call_site())) {
                            let match_str = format!("{name}{suffix}");
                            gen_inner.extend(quote!(
                                #match_str => <#special_type as DatatypeSetable>::set(&mut self.#name.#suffix, data)
                                    .map_err(|e| ContextSetError::Inner(#match_str, e, format!("{:?}", data))),
                            ));
                        }
                    } else {
                        let match_str = format!("{name}");
                        gen_inner.extend(quote!(
                            #match_str => <#ty as DatatypeSetable>::set(&mut self.#name, data)
                                .map_err(|e| ContextSetError::Inner(#match_str, e, format!("{:?}", data))),
                        ));
                    }
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let gen = quote!(
        impl #impl_generics crate::SetByName for #name #ty_generics #where_clause {
            fn set(&mut self, name: &str, data: &Datatype<'_>) -> Result<(), ContextSetError> {
                match name {
                    #gen_inner
                    _ => Err(ContextSetError::NameNotFound { name: name.to_owned() }),
                }
            }
        }
    );

    gen.into()
}
