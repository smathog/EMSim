use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{ItemFn, parse, parse_macro_input};

/// Practice attribute macro for injecting print statements
#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();
    let item_str = item.to_string();
    let mut input = parse_macro_input!(item as ItemFn);
    let mut statements = vec![
        syn::parse(quote!(println!("Attributes: {}", #attr_str);).into()).unwrap(),
        syn::parse(quote!(println!("Item: {}", #item_str);).into()).unwrap(),
    ];
    statements.append(&mut input.block.stmts);
    input.block.stmts = statements;
    input.into_token_stream().into()
}

/// Practice attribute macro for injecting print statements
#[proc_macro_attribute]
pub fn count_statements(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let count = input.block.stmts.len();
    input.block.stmts.push(syn::parse(quote!(println!("Num statements: {}", #count);).into()).unwrap());
    for i in 0..count {
        let statement = input.block.stmts[i].clone();
        let statement = quote!(#statement).to_string();
        input.block.stmts.push(syn::parse(quote!(println!("Statement {}: {:?}", #i,
            #statement);).into()).unwrap());
    }
    input.into_token_stream().into()
}
