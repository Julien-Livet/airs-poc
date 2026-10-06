use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr};

#[proc_macro_attribute]
pub fn primitive(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    match primitive_impl(attr, item) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error().into(),
    }
}

fn primitive_impl(
    attr: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let name = syn::parse::<LitStr>(attr)?;
    let function = syn::parse::<ItemFn>(item)?;

    let function_name = &function.sig.ident;

    let apply_name = syn::Ident::new(
        &format!("{}_apply", function_name),
        function_name.span(),
    );

    let entry_name = syn::Ident::new(
        &format!("{}_entry", function_name),
        function_name.span(),
    );

    let inputs = function
        .sig
        .inputs
        .iter()
        .map(|input| {
            match input {
                syn::FnArg::Typed(input) => {
                    type_tokens(&input.ty)
                }
                syn::FnArg::Receiver(_) => {
                    Err(syn::Error::new_spanned(
                        input,
                        "primitive functions cannot have self",
                    ))
                }
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let rust_inputs = function
        .sig
        .inputs
        .iter()
        .map(|input| {
            match input {
                syn::FnArg::Typed(input) => {
                    Ok(input.ty.clone())
                }
                syn::FnArg::Receiver(_) => {
                    Err(syn::Error::new_spanned(
                        input,
                        "primitive functions cannot have self",
                    ))
                }
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let rust_output = match &function.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &function.sig,
                "primitive function must have an output type",
            ));
        }

        syn::ReturnType::Type(_, ty) => {
            ty.clone()
        }
    };

    let argument_names = rust_inputs
        .iter()
        .enumerate()
        .map(|(index, _)| {
            syn::Ident::new(
                &format!("arg{}", index),
                proc_macro2::Span::call_site(),
            )
        })
        .collect::<Vec<_>>();

    let argument_bindings = rust_inputs
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let name = syn::Ident::new(
                &format!("arg{}", index),
                proc_macro2::Span::call_site(),
            );

            let index = syn::Index::from(index);

            quote! {
                let #name =
                    <#ty as crate::registry::FromValue>
                        ::from_value(&values[#index])?;
            }
        })
        .collect::<Vec<_>>();

    let output = match &function.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &function.sig,
                "primitive function must have an output type",
            ));
        }

        syn::ReturnType::Type(_, ty) => {
            type_tokens(ty)?
        }
    };

    let input_count = inputs.len();

    Ok(quote! {
        #function

        const _: &str = #name;
        const _: &str = stringify!(#function_name);
        const _: usize = #input_count;

        const _: crate::registry::Type = #output;

        const _: &[crate::registry::Type] = &[
            #(#inputs),*
        ];

        fn #apply_name(
            values: &[crate::registry::Value],
        ) -> Result<crate::registry::Value, String> {
            #(#argument_bindings)*

            let result = #function_name(
                #(#argument_names),*
            );

            Ok(
                <#rust_output as crate::registry::IntoValue>
                    ::into_value(result)
            )
        }

        #[linkme::distributed_slice(crate::registry::PRIMITIVES)]
        pub static #entry_name: crate::registry::PrimitiveEntry =
            crate::registry::PrimitiveEntry {
                name: #name,
                inputs: &[
                    #(#inputs),*
                ],
                output: #output,
                apply: #apply_name,
            };
    }
    .into())
}

fn type_tokens(
    ty: &syn::Type,
) -> syn::Result<proc_macro2::TokenStream> {
    let syn::Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "primitive type must be a named type",
        ));
    };

    let segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| {
            syn::Error::new_spanned(
                ty,
                "primitive type path is empty",
            )
        })?;

    let tokens = match segment.ident.to_string().as_str() {
        "Integer" => quote! {
            crate::registry::Type::Integer
        },

        "IntegerTuple" => quote! {
            crate::registry::Type::IntegerTuple
        },

        "Grid" => quote! {
            crate::registry::Type::Grid
        },

        "Boolean" => quote! {
            crate::registry::Type::Boolean
        },

        "Object" => quote! {
            crate::registry::Type::Object
        },

        "Cell" => quote! {
            crate::registry::Type::Cell
        },

        "Indices" => quote! {
            crate::registry::Type::Indices
        },

        other => {
            return Err(syn::Error::new_spanned(
                &segment.ident,
                format!(
                    "unsupported primitive type: {}",
                    other
                ),
            ));
        }
    };

    Ok(tokens)
}
