// astract-macro/src/codegen.rs - Code generation for RSX elements

use crate::parser::{RsxAttr, RsxAttrValue, RsxElement};
use proc_macro2::TokenStream;
use quote::quote;

/// Generate egui code from RSX element
pub fn generate_element(element: &RsxElement) -> TokenStream {
    let tag = &element.tag;
    let tag_str = tag.to_string();

    match tag_str.as_str() {
        "Label" => generate_label(element),
        "Button" => generate_button(element),
        "VStack" => generate_vstack(element),
        "HStack" => generate_hstack(element),
        _ => {
            // Unknown tag, generate error
            quote! {
                compile_error!(concat!("Unknown RSX tag: ", stringify!(#tag)));
            }
        }
    }
}

/// Generate code for <Label text="..." />
fn generate_label(element: &RsxElement) -> TokenStream {
    if let Some(text) = find_attr_value(element, "text") {
        quote! {
            ui.label(#text);
        }
    } else {
        quote! {
            ui.label("");
        }
    }
}

/// Generate code for <Button text="..." on_click={...} />
fn generate_button(element: &RsxElement) -> TokenStream {
    let text = find_attr_value(element, "text").unwrap_or_else(|| quote! { "Button" });

    // Check for on_click handler
    if let Some(on_click) = find_attr_value(element, "on_click") {
        quote! {
            if ui.button(#text).clicked() {
                (#on_click)();
            }
        }
    } else {
        quote! {
            ui.button(#text);
        }
    }
}

/// Generate code for <VStack>children</VStack>
fn generate_vstack(element: &RsxElement) -> TokenStream {
    let children = generate_children(&element.children);

    quote! {
        ui.vertical(|ui| {
            #children
        });
    }
}

/// Generate code for <HStack>children</HStack>
fn generate_hstack(element: &RsxElement) -> TokenStream {
    let children = generate_children(&element.children);

    quote! {
        ui.horizontal(|ui| {
            #children
        });
    }
}

/// Generate code for children nodes
fn generate_children(children: &[crate::parser::RsxNode]) -> TokenStream {
    use crate::parser::RsxNode;

    let child_code: Vec<TokenStream> = children
        .iter()
        .map(|child| match child {
            RsxNode::Element(el) => generate_element(el),
            RsxNode::Text(text) => {
                quote! { ui.label(#text); }
            }
        })
        .collect();

    quote! {
        #(#child_code)*
    }
}

/// Helper: Find attribute by name and return its value as TokenStream
fn find_attr_value(element: &RsxElement, name: &str) -> Option<TokenStream> {
    element
        .attrs
        .iter()
        .find(|attr| attr.name == name)
        .map(|attr| match &attr.value {
            RsxAttrValue::Literal(lit) => {
                let text = lit.value();
                quote! { #text }
            }
            RsxAttrValue::Expr(expr) => {
                quote! { #expr }
            }
        })
}

/// Helper: Find attribute by name (deprecated, use find_attr_value)
#[allow(dead_code)]
fn find_attr<'a>(element: &'a RsxElement, name: &str) -> Option<&'a RsxAttr> {
    element.attrs.iter().find(|attr| attr.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RsxElement;
    use quote::quote;

    #[test]
    fn test_generate_label() {
        let input = quote! { <Label text="Hello" /> };
        let element: RsxElement = syn::parse2(input).unwrap();
        let code = generate_element(&element);

        // Should generate: ui.label("Hello");
        assert!(code.to_string().contains("ui . label"));
        assert!(code.to_string().contains("Hello"));
    }

    #[test]
    fn test_generate_button() {
        let input = quote! { <Button text="Click Me" /> };
        let element: RsxElement = syn::parse2(input).unwrap();
        let code = generate_element(&element);

        // Should generate: ui.button("Click Me");
        assert!(code.to_string().contains("ui . button"));
        assert!(code.to_string().contains("Click Me"));
    }

    #[test]
    fn test_unknown_tag_error() {
        let input = quote! { <UnknownWidget /> };
        let element: RsxElement = syn::parse2(input).unwrap();
        let code = generate_element(&element);

        // Should generate compile error
        assert!(code.to_string().contains("compile_error"));
    }
}
