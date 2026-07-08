use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Fields, Type, parse_macro_input};

#[proc_macro_derive(Filter, attributes(filter))]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let filter_ident = input.ident;

    let params_ident = match parse_filter_params_type(&input.attrs) {
        Ok(params_ident) => params_ident,
        Err(error) => return error.to_compile_error().into(),
    };

    let filter_name = to_snake_case(&filter_ident.to_string());

    quote! {
        impl #filter_ident {
            pub const NAME: &'static str = #filter_name;

            pub const fn definition() -> ::ddot_core::filter::FilterDefinition {
                ::ddot_core::filter::FilterDefinition {
                    name: Self::NAME,
                    params: <#params_ident as ::ddot_core::filter::FilterParams>::PARAMS,
                }
            }
        }

        impl ::ddot_core::filter::Filter for #filter_ident {
            type Params = #params_ident;

            fn name(&self) -> &'static str {
                Self::NAME
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
    let input = parse_macro_input!(input as DeriveInput);

    let params_ident = input.ident;

    let fields = match input.data {
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

    let params = match build_params(fields) {
        Ok(params) => params,
        Err(error) => return error.to_compile_error().into(),
    };

    let defaults = params.iter().map(|param| {
        let field_ident = &param.ident;
        let default = param.constraints.default_expr();

        quote! {
            #field_ident: #default
        }
    });

    let definitions = params.iter().map(|param| {
        let field_name = param.ident.to_string();
        let kind = param.kind_tokens();
        let min = option_f32_tokens(&param.constraints.min);
        let max = option_f32_tokens(&param.constraints.max);
        let default = param.constraints.default_expr();

        quote! {
            ::ddot_core::filter::ParamDefinition {
                name: #field_name,
                kind: #kind,
                min: #min,
                max: #max,
                default: (#default) as f32,
            }
        }
    });

    let validations = params.iter().filter_map(|param| {
        let field_ident = &param.ident;
        let field_name = field_ident.to_string();
        let min = param.constraints.min.as_ref()?;
        let max = param.constraints.max.as_ref()?;

        Some(quote! {
            if self.#field_ident < #min
                || self.#field_ident > #max
            {
                return Err(
                    ::ddot_core::filter::ParamError::OutOfRange {
                        name: #field_name,
                        value: self.#field_ident.to_string(),
                        min: stringify!(#min).to_string(),
                        max: stringify!(#max).to_string(),
                    }
                );
            }
        })
    });

    quote! {
        impl ::std::default::Default for #params_ident {
            fn default() -> Self {
                Self {
                    #(#defaults,)*
                }
            }
        }

        impl ::ddot_core::filter::FilterParams for #params_ident {
            const PARAMS: &'static [::ddot_core::filter::ParamDefinition] = &[
                #(#definitions,)*
            ];

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

fn parse_filter_params_type(attrs: &[syn::Attribute]) -> syn::Result<syn::Ident> {
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
            syn::Error::new_spanned(attr, "expected #[filter(params = ParamsType)]")
        });
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "missing #[filter(params = ParamsType)]",
    ))
}

fn build_params(fields: Fields) -> syn::Result<Vec<ParamField>> {
    let mut params = Vec::new();

    let named_fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "FilterParams requires named fields",
            ));
        }
    };

    for field in named_fields {
        let field_ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(&field, "FilterParams requires named fields"))?;

        let constraints = parse_param_constraints(&field.attrs)?;
        let kind = ParamKind::from_type(&field.ty)?;

        params.push(ParamField {
            ident: field_ident,
            constraints,
            kind,
        });
    }

    Ok(params)
}

struct ParamField {
    ident: syn::Ident,
    constraints: ParamConstraints,
    kind: ParamKind,
}

impl ParamField {
    fn kind_tokens(&self) -> proc_macro2::TokenStream {
        match self.kind {
            ParamKind::Int => quote!(::ddot_core::filter::ParamType::Int),
            ParamKind::Float => quote!(::ddot_core::filter::ParamType::Float),
        }
    }
}

enum ParamKind {
    Int,
    Float,
}

impl ParamKind {
    fn from_type(ty: &Type) -> syn::Result<Self> {
        let Type::Path(path) = ty else {
            return Err(syn::Error::new_spanned(
                ty,
                "param fields must be integer or floating point primitives",
            ));
        };

        let Some(segment) = path.path.segments.last() else {
            return Err(syn::Error::new_spanned(
                ty,
                "param fields must be integer or floating point primitives",
            ));
        };

        match segment.ident.to_string().as_str() {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => Ok(Self::Int),
            "f32" | "f64" => Ok(Self::Float),
            _ => Err(syn::Error::new_spanned(
                ty,
                "unsupported param type; expected an integer or floating point primitive",
            )),
        }
    }
}

struct ParamConstraints {
    min: Option<Expr>,
    max: Option<Expr>,
    default: Option<Expr>,
}

impl Default for ParamConstraints {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            default: None,
        }
    }
}

impl ParamConstraints {
    fn default_expr(&self) -> &Expr {
        self.default
            .as_ref()
            .expect("default is required before code generation")
    }
}

fn parse_param_constraints(attrs: &[syn::Attribute]) -> syn::Result<ParamConstraints> {
    let mut constraints = ParamConstraints::default();

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
                constraints.default = Some(value.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported param attribute"))
            }
        })?;
    }

    if constraints.default.is_none() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "param attributes must include default = ...",
        ));
    }

    Ok(constraints)
}

fn option_f32_tokens(expr: &Option<Expr>) -> proc_macro2::TokenStream {
    match expr {
        Some(expr) => quote!(Some((#expr) as f32)),
        None => quote!(None),
    }
}

fn to_snake_case(name: &str) -> String {
    let mut output = String::new();

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
