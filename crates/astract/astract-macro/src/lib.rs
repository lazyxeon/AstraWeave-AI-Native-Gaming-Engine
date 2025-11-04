use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

mod codegen;
mod parser;

use codegen::generate_element;
use parser::RsxElement;

/// RSX macro - expands JSX-like syntax to egui widget calls
///
/// Day 1: String literals: rsx!("Hello") → ui.label("Hello");
/// Day 2: Tag syntax: rsx!(<Label text="Hello" />) → ui.label("Hello");
///
/// Example:
/// ```ignore
/// rsx!(<Label text="Hello, world!" />);
/// ```
///
/// Expands to:
/// ```ignore
/// ui.label("Hello, world!");
/// ```
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    // Try parsing as RsxElement first (Day 2: tag syntax)
    if let Ok(element) = syn::parse::<RsxElement>(input.clone()) {
        let code = generate_element(&element);
        return TokenStream::from(code);
    }

    // Fallback to Day 1: string literal parsing
    let lit = parse_macro_input!(input as LitStr);
    let text = lit.value();

    let output = quote! {
        ui.label(#text);
    };

    TokenStream::from(output)
}
