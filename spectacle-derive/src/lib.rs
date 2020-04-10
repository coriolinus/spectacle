use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma,
    DeriveInput, Fields, GenericParam, Generics, Ident, Index, Variant,
};

#[proc_macro_derive(Spectacle)]
pub fn derive_spectacle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);

    match input.data {
        syn::Data::Struct(data) => impl_introspect_struct(&name, &generics, &data.fields),
        syn::Data::Enum(data) => impl_introspect_enum(&name, &generics, &data.variants),
        _ => unimplemented!(),
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
    let mut n = 0;
    while generics.params.iter().any(|param| match param {
        GenericParam::Type(param) => param.ident == ident,
        GenericParam::Const(param) => param.ident == ident,
        GenericParam::Lifetime(param) => param.lifetime.ident == ident,
    }) {
        ident = Ident::new(&format!("F{}", n), generics.span());
        n += 1;
    }
    ident
}

fn impl_introspect_struct(
    name: &Ident,
    generics: &Generics,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let f = create_generic_ident(&generics);
    let recurse = recurse_fields(fields);

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

fn impl_introspect_enum(
    name: &Ident,
    generics: &Generics,
    variants: &Punctuated<Variant, Comma>,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let f = create_generic_ident(&generics);

    quote! {
        impl #impl_generics Introspect for #name #ty_generics #where_clause
        {
            fn introspect_from<#f>(&self, breadcrumbs: Breadcrumbs, visit: #f)
            where
                #f: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);

                match self {
                    unimplemented!("figure out how to do variants gracefully")
                }
            }
        }
    }
}

// TODO: more fine-grained control of field visibility somehow
// for now, we'll visit all fields, even private ones
fn recurse_fields(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => quote! {},
        Fields::Named(fields) => {
            let recurse = fields.named.iter().map(|field| {
                let name = field.ident.clone().expect("named fields have names");
                let name_lit = syn::LitStr::new(&format!("{}", name), field.span());

                quote_spanned! {field.span() =>  {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::Field(#name_lit));
                    spectacle::Introspect::introspect_from(self.#name, breadcrumbs, &visit);
                }}
            });

            quote! { #( #recurse )* }
        }
        Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().enumerate().map(|(i, field)| {
                let idx = Index::from(i);

                quote_spanned! {field.span() =>  {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::TupleIndex(#i));
                    spectacle::Introspect::introspect_from(self.#idx, breadcrumbs, &visit);
                }}
            });

            quote! { #( #recurse )* }
        }
    }
}
