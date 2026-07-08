use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    DeriveInput,
    Expr,
    Fields,
};


#[proc_macro_derive(Filter, attributes(filter))]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    let input =
        parse_macro_input!(input as DeriveInput);

    let filter_ident =
        input.ident;

    let params_ident =
        match parse_filter_params_type(&input.attrs) {
            Ok(params_ident) => params_ident,
            Err(error) => return error.to_compile_error().into(),
        };

    let filter_name =
        to_snake_case(&filter_ident.to_string());

    quote! {
        impl ::ddot_core::filter::Filter for #filter_ident {
            type Params = #params_ident;

            fn name(&self) -> &'static str {
                #filter_name
            }

            fn apply(
                &self,
                image: &mut ::ddot_core::image::Image,
                params: &Self::Params,
            ) {
                #filter_ident::apply(
                    self,
                    image,
                    params,
                );
            }
        }
    }
    .into()
}


#[proc_macro_derive(FilterParams, attributes(param))]
pub fn derive_filter_params(input: TokenStream) -> TokenStream {
    let input =
        parse_macro_input!(input as DeriveInput);

    let params_ident =
        input.ident;

    let fields =
        match input.data {
            syn::Data::Struct(data) => data.fields,
            _ => {
                return syn::Error::new_spanned(
                    params_ident,
                    "FilterParams can only be derived for structs",
                )
                .to_compile_error()
                .into();
            }
        };

    let validations =
        match build_param_validations(fields) {
            Ok(validations) => validations,
            Err(error) => return error.to_compile_error().into(),
        };

    quote! {
        impl ::ddot_core::filter::FilterParams for #params_ident {
            fn validate(
                &self,
            ) -> Result<(), ::ddot_core::filter::ParamError> {
                #(#validations)*

                Ok(())
            }
        }
    }
    .into()
}


fn parse_filter_params_type(
    attrs: &[syn::Attribute],
) -> syn::Result<syn::Ident> {
    for attr in attrs {
        if !attr.path().is_ident("filter") {
            continue;
        }

        let mut params_ident = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("params") {
                let value = meta.value()?;
                params_ident = Some(value.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported filter attribute"))
            }
        })?;

        return params_ident.ok_or_else(|| {
            syn::Error::new_spanned(
                attr,
                "expected #[filter(params = ParamsType)]",
            )
        });
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "missing #[filter(params = ParamsType)]",
    ))
}


fn build_param_validations(
    fields: Fields,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut validations =
        Vec::new();

    let named_fields =
        match fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "FilterParams requires named fields",
                ));
            }
        };

    for field in named_fields {
        let field_ident =
            field.ident.clone().ok_or_else(|| {
                syn::Error::new_spanned(
                    &field,
                    "FilterParams requires named fields",
                )
            })?;

        let field_name =
            field_ident.to_string();

        let constraints =
            parse_param_constraints(&field.attrs)?;

        if let Some(min) = constraints.min {
            validations.push(quote! {
                if self.#field_ident < #min {
                    return Err(::ddot_core::filter::ParamError::OutOfRange {
                        name: #field_name,
                        value: self.#field_ident.to_string(),
                    });
                }
            });
        }

        if let Some(max) = constraints.max {
            validations.push(quote! {
                if self.#field_ident > #max {
                    return Err(::ddot_core::filter::ParamError::OutOfRange {
                        name: #field_name,
                        value: self.#field_ident.to_string(),
                    });
                }
            });
        }
    }

    Ok(validations)
}


#[derive(Default)]
struct ParamConstraints {
    min: Option<Expr>,
    max: Option<Expr>,
}


fn parse_param_constraints(
    attrs: &[syn::Attribute],
) -> syn::Result<ParamConstraints> {
    let mut constraints =
        ParamConstraints::default();

    for attr in attrs {
        if !attr.path().is_ident("param") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("min") {
                let value = meta.value()?;
                constraints.min = Some(value.parse()?);
                Ok(())
            } else if meta.path.is_ident("max") {
                let value = meta.value()?;
                constraints.max = Some(value.parse()?);
                Ok(())
            } else if meta.path.is_ident("default") {
                let value = meta.value()?;
                let _: Expr = value.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported param attribute"))
            }
        })?;
    }

    Ok(constraints)
}


fn to_snake_case(
    name: &str,
) -> String {
    let mut output =
        String::new();

    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() {
            if index > 0 {
                output.push('_');
            }

            for lower in character.to_lowercase() {
                output.push(lower);
            }
        } else {
            output.push(character);
        }
    }

    output
}
