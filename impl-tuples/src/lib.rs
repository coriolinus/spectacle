//! Don't use this crate. It's intended for use in exactly one place: spectacle.
//! It does exactly one thing, for spectacle. It is likely to break in other contexts.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

struct UpTo {
    value: usize,
}

impl Parse for UpTo {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit: syn::LitInt = input.parse()?;
        let value = lit.base10_parse::<usize>()?;
        Ok(UpTo { value })
    }
}

#[proc_macro]
pub fn impl_tuples(tokens: TokenStream) -> TokenStream {
    let up_to = parse_macro_input!(tokens as UpTo);

    let mut out = quote! {};

    for arity in 0..=up_to.value {
        let t_n = (0..arity)
            .map(|n| format_ident!("T{}", n))
            .collect::<Vec<_>>();
        let idx = (0..arity).map(syn::Index::from).collect::<Vec<_>>();

        out = quote! {
            #out

            impl<#(#t_n),*> Spectacle for (#(#t_n,)*)
            where
                #(
                    #t_n: 'static + Spectacle,
                )*
            {
                fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
                where
                    F: Fn(&Breadcrumbs, &dyn Any),
                {
                    visit(&breadcrumbs, self);

                    #({
                        let mut breadcrumbs = breadcrumbs.clone();
                        breadcrumbs.push_back(Breadcrumb::TupleIndex(#idx));
                        self.#idx.introspect_from(breadcrumbs, &visit);
                    })*
                }
            }
        }
    }

    // eprintln!("impl_tuples:\n{}", out);

    out.into()
}
