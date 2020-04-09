use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, LitInt};

struct UpTo {
    value: u8,
}

impl Parse for UpTo {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit: LitInt = input.parse()?;
        let value = lit.base10_parse::<u8>()?;
        Ok(UpTo { value })
    }
}

#[proc_macro]
pub fn impl_tuples(tokens: TokenStream) -> TokenStream {
    let up_to = parse_macro_input!(tokens as UpTo);

    let mut out = quote! {};

    for arity in 1..=up_to.value {
        let t_n = (0..arity)
            .map(|n| format_ident!("T{}", n))
            .collect::<Vec<_>>();
        let idx = (0..arity).collect::<Vec<_>>();

        out = quote! {
            #out

            impl<#(#t_n),*> Spectacle for (#(#t_n),*)
            where
                #(
                    #t_n: 'static + Spectacle,
                ),*
            {
                fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
                where
                    F: Fn(&Breadcrumbs, &dyn Any),
                {
                    visit(&breadcrumbs, self);

                    #(
                        let mut breadcrumbs = breadcrumbs.clone();
                        breadcrumbs.push_back(Breadcrumb::TupleIndex(#idx));
                        self.$idx.introspect_from(breadcrumbs, &visit);
                    )*
                }
            }
        }
    }

    out.into()
}
