use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use std::borrow::Borrow;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma,
    DeriveInput, Fields, GenericParam, Generics, Ident, Index, Type, Variant,
};

#[proc_macro_derive(Spectacle)]
#[proc_macro_error]
pub fn derive_spectacle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);

    match input.data {
        syn::Data::Struct(data) => impl_introspect_struct(&name, &generics, &data.fields),
        syn::Data::Enum(data) => impl_introspect_enum(&name, &generics, &data.variants),
        syn::Data::Union(_) => {
            emit_error!(
                name.span(),
                "Spectacle can only be derived for structs and enums"
            );

            TokenStream::new()
        }
    }
    .into()
}

// Add a bound `T: 'static + Introspect` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!('static + spectacle::Introspect));
        }
    }
    generics
}

// Create an unused generic identifier
fn create_generic_ident(generics: &Generics) -> Ident {
    let mut ident = Ident::new("F", generics.span());
    let mut n: u8 = 0;
    while generics.params.iter().any(|param| match param {
        GenericParam::Type(param) => param.ident == ident,
        GenericParam::Const(param) => param.ident == ident,
        GenericParam::Lifetime(param) => param.lifetime.ident == ident,
    }) {
        ident = Ident::new(&format!("F{}", n), generics.span());
        n += 1;
        if n == std::u8::MAX {
            emit_error!(
                generics,
                "could not generate an appropriate unused type parameter";
                note = "#[derive(Spectacle)] must be able to generate an unused type parameter F or F{n} where n is in the u16 range";
                help = "consider removing the type parameter F from your list of generic parameters";
            );
        }
    }
    ident
}

fn impl_introspect_struct(name: &Ident, generics: &Generics, fields: &Fields) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let f = create_generic_ident(&generics);
    let recurse = recurse_fields(fields, |field_id| quote!(self.#field_id)).unwrap_or_default();

    quote! {
        impl #impl_generics Introspect for #name #ty_generics #where_clause
        {
            fn introspect_from<#f>(&self, breadcrumbs: Breadcrumbs, visit: #f)
            where
                #f: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);

                #recurse
            }
        }
    }
}

// TODO: more fine-grained control of field visibility somehow
// for now, we'll visit all fields, even private ones
fn recurse_fields<Accessor>(fields: &Fields, access: Accessor) -> Option<TokenStream>
where
    Accessor: Fn(Box<dyn quote::ToTokens>) -> TokenStream,
{
    match fields {
        Fields::Unit => None,
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|field| {
                let name = field.ident.clone().expect("named fields have names");
                let name_lit = syn::LitStr::new(&format!("{}", name), field.span());
                let field = access(Box::new(name));

                quote! {{
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::Field(#name_lit));
                    spectacle::Introspect::introspect_from(#field, breadcrumbs, &visit);
                }}
            });

            Some(quote! { #( #recurse )* })
        }
        Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().enumerate().map(|(i, field)| {
                let idx = Index::from(i);
                let field = access(Box::new(idx));

                quote! {{
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::TupleIndex(#i));
                    spectacle::Introspect::introspect_from(#field, breadcrumbs, &visit);
                }}
            });

            Some(quote! { #( #recurse )* })
        }
    }
}

fn impl_introspect_enum(
    name: &Ident,
    generics: &Generics,
    variants: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let f = create_generic_ident(&generics);
    let recurse = recurse_variants(variants);

    quote! {
        impl #impl_generics Introspect for #name #ty_generics #where_clause
        {
            fn introspect_from<#f>(&self, breadcrumbs: Breadcrumbs, visit: #f)
            where
                #f: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);

                match self {
                    #( #recurse ),*
                }
            }
        }
    }
}

// form an ident to refer to an unnamed type:
// lowercase + append index
fn type_var<T>(t: T, n: Option<usize>) -> Ident
where
    T: Borrow<Type> + Spanned,
{
    let ident = match t.borrow() {
        Type::Array(t) => return format_ident!("{}s", type_var(t.elem.borrow(), n)),
        Type::Slice(t) => return format_ident!("{}s", type_var(t.elem.borrow(), n)),
        Type::Group(t) => return type_var(t.elem.borrow(), n),
        Type::Paren(t) => return type_var(t.elem.borrow(), n),
        Type::Ptr(t) => return type_var(t.elem.borrow(), n),
        Type::Reference(t) => return type_var(t.elem.borrow(), n),
        Type::Path(t) => t
            .path
            .segments
            .last()
            .expect("type paths should not be empty")
            .ident
            .clone(),
        Type::Tuple(t) => {
            let names = t
                .elems
                .iter()
                .map(|tt| type_var(tt, None))
                .map(|id| id.to_string())
                .collect::<Vec<_>>();
            format_ident!("{}", names.join("_"))
        }
        _ => {
            emit_error!(
                t.span(),
                "cannot create appropriate type variable for this type"
            );
            format_ident!("_0")
        }
    };
    match n {
        None => ident,
        Some(n) => format_ident!("{}{}", ident.to_string().to_lowercase(), n),
    }
}

fn recurse_variants(variants: &Punctuated<Variant, Comma>) -> Vec<TokenStream> {
    variants
        .iter()
        .filter_map(|variant| {
            if variant.fields.is_empty() {
                return None;
            }

            let name = &variant.ident;
            let field_name = variant.fields.iter().enumerate().map(|(idx, field)| {
                let idx = if let Fields::Unnamed(_) = variant.fields {
                    Some(idx)
                } else {
                    None
                };
                type_var(&field.ty, idx)
            });
            let (open_brace, close_brace) = match variant.fields {
                Fields::Named(_) => ("{", "}"),
                Fields::Unnamed(_) => ("(", ")"),
                _ => unreachable!(),
            };
            let recurse = recurse_fields(&variant.fields, unimplemented!());

            Some(quote! {
                #name #open_brace #( #field_name ),* #close_brace => #recurse
            })
        })
        .collect()
}
