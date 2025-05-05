use proc_macro2::TokenStream;

use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Item;
use syn::ItemMod;
use syn::parse_quote;

pub(crate) fn mcrl3_derive_terms_impl(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut ast: ItemMod = syn::parse2(input.clone()).expect("mcrl3_term can only be applied to a module");

    if let Some((_, content)) = &mut ast.content {
        // Generated code blocks are added to this list.
        let mut added = vec![];

        for item in content.iter_mut() {
            match item {
                Item::Struct(object) => {
                    // If the struct is annotated with term we process it as a term.
                    if let Some(attr) = object.attrs.iter().find(|attr| attr.meta.path().is_ident("mcrl3_term")) {
                        // The #term(assertion) annotation must contain an assertion
                        let assertion = match attr.parse_args::<syn::Ident>() {
                            Ok(assertion) => {
                                let assertion_msg = format!("{assertion}");
                                quote!(
                                    debug_assert!(#assertion(&term), "Term {:?} does not satisfy {}", term, #assertion_msg)
                                )
                            }
                            Err(_x) => {
                                quote!()
                            }
                        };

                        // Add the expected derive macros to the input struct.
                        object
                            .attrs
                            .push(parse_quote!(#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]));

                        // ALL structs in this module must contain the term.
                        assert!(
                            object.fields.iter().any(|field| {
                                if let Some(name) = &field.ident {
                                    name == "term"
                                } else {
                                    false
                                }
                            }),
                            "The struct {} in mod {} has no field 'term: ATerm'",
                            object.ident,
                            ast.ident
                        );

                        let name = format_ident!("{}", object.ident);

                        // Add a <name>Ref struct that contains the ATermRef<'a> and
                        // the implementation and both protect and borrow. Also add
                        // the conversion from and to an ATerm.
                        let name_ref = format_ident!("{}Ref", object.ident);
                        let generated: TokenStream = quote!(
                            impl #name {
                                pub fn copy<'a>(&'a self) -> #name_ref<'a> {
                                    self.term.copy().into()
                                }
                            }

                            impl From<ATerm> for #name {
                                fn from(term: ATerm) -> #name {
                                    #assertion;
                                    #name {
                                        term
                                    }
                                }
                            }

                            impl Into<ATerm> for #name {
                                fn into(self) -> ATerm {
                                    self.term
                                }
                            }

                            impl Deref for #name {
                                type Target = ATerm;

                                fn deref(&self) -> &Self::Target {
                                    &self.term
                                }
                            }

                            impl Borrow<ATerm> for #name {
                                fn borrow(&self) -> &ATerm {
                                    &self.term
                                }
                            }

                            impl Markable for #name {
                                fn mark(&self, marker: &mut Marker) {
                                    self.term.mark(marker);
                                }

                                fn contains_term(&self, term: &ATermRef<'_>) -> bool {
                                    &self.term.copy() == term
                                }

                                fn len(&self) -> usize {
                                    1
                                }
                            }

                            impl<'a, 'b> Term<'a, 'b> for #name where 'b: 'a {
                                delegate! {
                                    to self.term {
                                        fn protect(&self) -> ATerm;
                                        fn arg(&'b self, index: usize) -> ATermRef<'a>;
                                        fn arguments(&'b self) -> ATermArgs<'a>;
                                        fn copy(&'b self) -> ATermRef<'a>;
                                        fn get_head_symbol(&'b self) -> SymbolRef<'a>;
                                        fn iter(&'b self) -> TermIterator<'a>;
                                        fn index(&self) -> usize;
                                        fn shared(&self) -> &ATermIndex;
                                    }
                                }
                            }

                            #[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
                            pub struct #name_ref<'a> {
                                pub(crate) term: ATermRef<'a>
                            }

                            impl<'a> #name_ref<'a> {
                                pub fn copy<'b>(&'b self) -> #name_ref<'b> {
                                    self.term.copy().into()
                                }

                                pub fn protect(&self) -> #name {
                                    self.term.protect().into()
                                }
                            }

                            impl<'a> From<ATermRef<'a>> for #name_ref<'a> {
                                fn from(term: ATermRef<'a>) -> #name_ref<'a> {
                                    #assertion;
                                    #name_ref {
                                        term
                                    }
                                }
                            }

                            impl<'a> Into<ATermRef<'a>> for #name_ref<'a> {
                                fn into(self) -> ATermRef<'a> {
                                    self.term
                                }
                            }

                            impl<'a> Term<'a, '_> for #name_ref<'a> {
                                delegate! {
                                    to self.term {
                                        fn protect(&self) -> ATerm;
                                        fn arg(&self, index: usize) -> ATermRef<'a>;
                                        fn arguments(&self) -> ATermArgs<'a>;
                                        fn copy(&self) -> ATermRef<'a>;
                                        fn get_head_symbol(&self) -> SymbolRef<'a>;
                                        fn iter(&self) -> TermIterator<'a>;
                                        fn index(&self) -> usize;
                                        fn shared(&self) -> &ATermIndex;
                                    }
                                }
                            }

                            impl<'a> Borrow<ATermRef<'a>> for #name_ref<'a> {
                                fn borrow(&self) -> &ATermRef<'a> {
                                    &self.term
                                }
                            }

                            impl<'a> Markable for #name_ref<'a> {
                                fn mark(&self, marker: &mut Marker) {
                                    self.term.mark(marker);
                                }

                                fn contains_term(&self, term: &ATermRef<'_>) -> bool {
                                    &self.term == term
                                }

                                fn len(&self) -> usize {
                                    1
                                }
                            }

                            impl Transmutable for #name_ref<'static> {
                                type Target<'a> = #name_ref<'a>;

                                fn transmute_lifetime<'a>(&self) -> &'a Self::Target<'a> {
                                    unsafe { transmute::<&Self, &'a #name_ref<'a>>(self) }
                                }

                                fn transmute_lifetime_mut<'a>(&mut self) -> &'a mut Self::Target<'a> {
                                    unsafe { transmute::<&mut Self, &'a mut #name_ref<'a>>(self) }
                                }
                            }
                        );

                        added.push(Item::Verbatim(generated));
                    }
                }
                Item::Impl(implementation) => {
                    if !implementation
                        .attrs
                        .iter()
                        .any(|attr| attr.meta.path().is_ident("mcrl3_ignore"))
                    {
                        // Duplicate the implementation for the ATermRef struct that is generated above.
                        let mut ref_implementation = implementation.clone();

                        // Remove ignore functions
                        ref_implementation.items.retain(|item| match item {
                            syn::ImplItem::Fn(func) => {
                                !func.attrs.iter().any(|attr| attr.meta.path().is_ident("mcrl3_ignore"))
                            }
                            _ => true,
                        });

                        if let syn::Type::Path(path) = ref_implementation.self_ty.as_ref() {
                            // Build an identifier TestRef<'_>
                            let name_ref = format_ident!("{}Ref", path.path.get_ident().unwrap());
                            let path = parse_quote!(#name_ref <'_>);

                            ref_implementation.self_ty = Box::new(syn::Type::Path(syn::TypePath { qself: None, path }));

                            added.push(Item::Verbatim(ref_implementation.into_token_stream()));
                        }
                    }
                }
                _ => {
                    // Ignore the rest.
                }
            }
        }

        content.append(&mut added);
    }

    // Hand the output tokens back to the compiler
    ast.into_token_stream()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_macro() {
        let input = "
            mod anything {

                #[mcrl3_term(test)]
                #[derive(Debug)]
                struct Test {
                    term: ATerm,
                }

                impl Test {
                    fn a_function() {

                    }
                }
            }
        ";

        let tokens = TokenStream::from_str(input).unwrap();
        let result = mcrl3_derive_terms_impl(TokenStream::default(), tokens);

        println!("{result}");
    }
}
