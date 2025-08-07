use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, ItemStruct, Token};

#[proc_macro_derive(InterfaceImpl, attributes(interface, versions))]
pub fn derive_interface_impl(tokens: TokenStream) -> TokenStream {
    let s = parse_macro_input!(tokens as ItemStruct);
    let name = s.ident;

    // Parse the versions attribute
    let versions_attr = s
        .attrs
        .iter()
        .find(|a| a.style == syn::AttrStyle::Outer && a.meta.path().is_ident("versions"))
        .expect("missing versions attribute");

    let versions = versions_attr
        .meta
        .require_list()
        .expect("versions attribute should be a list")
        .parse_args_with(Punctuated::<syn::LitInt, Token![,]>::parse_separated_nonempty)
        .expect("parsing versions failed");

    // Parse the interface attribute
    let interface_attr = s
        .attrs
        .into_iter()
        .find(|a| a.style == syn::AttrStyle::Outer && a.meta.path().is_ident("interface"))
        .expect("missing interface attribute");

    let interface = interface_attr
        .meta
        .require_name_value()
        .expect("parsing interface failed");

    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(interface),
        ..
    }) = &interface.value
    else {
        panic!("expected string literal for interface");
    };
    let interface = interface.value();

    // Generate version-specific identifiers
    let interface_versions: Vec<syn::Ident> = versions
        .iter()
        .map(|v| {
            let num: u8 = v.base10_parse().unwrap();
            format_ident!("{interface}_{num:03}")
        })
        .collect();

    let version_variants: Vec<syn::Ident> =
        versions.iter().map(|v| format_ident!("V{v}")).collect();

    // Generate vtable wrapper variants
    let wrapped_variants =
        interface_versions
            .iter()
            .zip(&version_variants)
            .map(|(interface, variant)| {
                quote! {
                    #variant(VtableWrapper<vr::#interface, #name>)
                }
            });

    // Generate version matching arms
    let get_vtable_arms = interface_versions
        .iter()
        .zip(&version_variants)
        .enumerate()
        .map(|(index, (interface, variant))| {
            quote! {
                x if x == vr::#interface::VERSION => {
                    Some(Box::new(move |this: &std::sync::Arc<Self>| {
                        let vtable = this.vtables.0[#index]
                            .get_or_init(|| {
                                WrappedVtable::#variant(
                                    <Self as Inherits<vr::#interface>>::new_wrapped(this)
                                )
                            });

                        let WrappedVtable::#variant(vtable) = vtable else {
                            unreachable!("vtable version mismatch")
                        };
                        &vtable.base as *const vr::#interface as *mut std::ffi::c_void
                    }))
                }
            }
        });

    let num_versions = versions.len();
    let mod_name = format_ident!("{}_gen", name.to_string().to_lowercase());

    let output = quote! {
        use #mod_name::Vtables;

        mod #mod_name {
            use super::*;
            use std::sync::OnceLock;
            use std::ffi::CStr;
            use crate::{VtableWrapper, Inherits};
            use crate::vr;

            pub struct Vtables([OnceLock<WrappedVtable>; #num_versions]);

            impl Default for Vtables {
                fn default() -> Self {
                    Self(std::array::from_fn(|_| OnceLock::new()))
                }
            }

            enum WrappedVtable {
                #(#wrapped_variants),*
            }

            unsafe impl Sync for WrappedVtable {}
            unsafe impl Send for WrappedVtable {}

            impl crate::InterfaceImpl for super::#name {
                fn supported_versions() -> &'static [&'static CStr] {
                    &[
                        #(vr::#interface_versions::VERSION),*
                    ]
                }

                fn get_version(version: &std::ffi::CStr) -> Option<Box<dyn FnOnce(&std::sync::Arc<Self>) -> *mut std::ffi::c_void>> {
                    match version {
                        #(#get_vtable_arms)*
                        _ => None
                    }
                }
            }
        }
    };

    output.into()
}
