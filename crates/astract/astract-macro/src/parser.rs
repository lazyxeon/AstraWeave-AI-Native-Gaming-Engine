// astract-macro/src/parser.rs - RSX syntax parser

use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, LitStr, Result, Token,
};

/// Represents a single RSX element: `<Tag attr="value" />`
#[derive(Debug, Clone)]
pub struct RsxElement {
    pub tag: Ident,
    pub attrs: Vec<RsxAttr>,
    pub children: Vec<RsxNode>,
    #[allow(dead_code)]
    pub self_closing: bool,
}

/// Represents an attribute: `attr="value"` or `attr={expr}`
#[derive(Debug, Clone)]
pub struct RsxAttr {
    pub name: Ident,
    pub value: RsxAttrValue,
}

/// Attribute value can be string literal or code block
#[derive(Clone)]
pub enum RsxAttrValue {
    Literal(LitStr),
    Expr(syn::Expr), // Day 3: Code blocks like {|| count += 1}
}

impl std::fmt::Debug for RsxAttrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsxAttrValue::Literal(lit) => write!(f, "Literal(\"{}\")", lit.value()),
            RsxAttrValue::Expr(_) => write!(f, "Expr(...)"),
        }
    }
}

/// RSX node can be element or text
#[derive(Clone)]
pub enum RsxNode {
    Element(RsxElement),
    Text(String), // Changed from LitStr to String for Debug
}

impl std::fmt::Debug for RsxNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsxNode::Element(el) => write!(f, "Element({:?})", el),
            RsxNode::Text(text) => write!(f, "Text(\"{}\")", text),
        }
    }
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse opening tag: <Tag
        input.parse::<Token![<]>()?;
        let tag: Ident = input.parse()?;

        // Parse attributes
        let mut attrs = Vec::new();
        while !input.peek(Token![/]) && !input.peek(Token![>]) {
            attrs.push(input.parse()?);
        }

        // Parse children and determine if self-closing
        let mut children: Vec<RsxNode> = Vec::new();
        let self_closing = if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            true
        } else {
            // Parse closing tag: >
            input.parse::<Token![>]>()?;

            // Parse children (recursive)
            while !input.peek(Token![<]) || !input.peek2(Token![/]) {
                // Try parsing child element
                if input.peek(Token![<]) && !input.peek2(Token![/]) {
                    let child_element: RsxElement = input.parse()?;
                    children.push(RsxNode::Element(child_element));
                } else if input.peek(LitStr) {
                    // Parse text node
                    let text: LitStr = input.parse()?;
                    children.push(RsxNode::Text(text.value()));
                } else {
                    // No more children, break
                    break;
                }
            }

            // Parse closing tag: </Tag>
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            let closing_tag: Ident = input.parse()?;
            input.parse::<Token![>]>()?;

            // Validate matching tags
            if tag != closing_tag {
                return Err(Error::new(
                    closing_tag.span(),
                    format!(
                        "Closing tag `{}` does not match opening tag `{}`",
                        closing_tag, tag
                    ),
                ));
            }

            false
        };

        Ok(RsxElement {
            tag,
            attrs,
            children,
            self_closing,
        })
    }
}

impl Parse for RsxAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        // Parse value: string literal or code block {...}
        let value = if input.peek(LitStr) {
            RsxAttrValue::Literal(input.parse()?)
        } else if input.peek(syn::token::Brace) {
            // Parse code block: {expr}
            let content;
            syn::braced!(content in input);
            let expr: syn::Expr = content.parse()?;
            RsxAttrValue::Expr(expr)
        } else {
            return Err(Error::new(
                input.span(),
                "Expected string literal or code block {...}",
            ));
        };

        Ok(RsxAttr { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_parse_self_closing_tag() {
        let input = quote! { <Label text="Hello" /> };
        let element: RsxElement = syn::parse2(input).unwrap();

        assert_eq!(element.tag.to_string(), "Label");
        assert!(element.self_closing);
        assert_eq!(element.attrs.len(), 1);
        assert_eq!(element.attrs[0].name.to_string(), "text");
    }

    #[test]
    fn test_parse_tag_with_closing() {
        let input = quote! { <Button></Button> };
        let element: RsxElement = syn::parse2(input).unwrap();

        assert_eq!(element.tag.to_string(), "Button");
        assert!(!element.self_closing);
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let input = quote! { <Input value="test" placeholder="Enter text" /> };
        let element: RsxElement = syn::parse2(input).unwrap();

        assert_eq!(element.attrs.len(), 2);
        assert_eq!(element.attrs[0].name.to_string(), "value");
        assert_eq!(element.attrs[1].name.to_string(), "placeholder");
    }

    #[test]
    #[should_panic(expected = "does not match")]
    fn test_mismatched_tags() {
        let input = quote! { <VStack></HStack> };
        let _element: RsxElement = syn::parse2(input).unwrap();
    }
}
