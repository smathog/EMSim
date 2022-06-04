use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ImplItem, ImplItemMethod, ItemFn, ItemImpl};

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
    input
        .block
        .stmts
        .push(syn::parse(quote!(println!("Num statements: {}", #count);).into()).unwrap());
    for i in 0..count {
        let statement = input.block.stmts[i].clone();
        let statement = quote!(#statement).to_string();
        input.block.stmts.push(
            syn::parse(
                quote!(println!("Statement {}: {:?}", #i,
            #statement);)
                .into(),
            )
            .unwrap(),
        );
    }
    input.into_token_stream().into()
}

/// Appends an associated const into a impl block that counts the number of methods
/// in that impl block
#[proc_macro_attribute]
pub fn append_method_count(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let count = input
        .items
        .iter()
        .filter(|item| match item {
            &&ImplItem::Method(_) => true,
            _ => false,
        })
        .count();
    input
        .items
        .push(syn::parse(quote!(const METHOD_COUNT: usize = #count;).into()).unwrap());
    input.into_token_stream().into()
}

/// Injects a function into the impl block that prints the name of the functions and their place (0-
/// indexed) in order in the impl block
#[proc_macro_attribute]
pub fn inject_method_counter(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    // Get list of methods
    let methods = input
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method.sig.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Create method to append to impl block that will display the available methods to call
    let mut counter_method: ImplItemMethod = syn::parse(
        quote!(
            pub fn list_methods() {
                let mut methods = Vec::new();
            }
        )
        .into(),
    )
    .unwrap();
    for s in methods {
        counter_method
            .block
            .stmts
            .push(syn::parse(quote!(methods.push(#s);).into()).unwrap())
    }
    counter_method.block.stmts.push(
        syn::parse(
            quote!(for (i, s) in methods.into_iter().enumerate() {
                println!("Method {}: {}", i, s);
            })
            .into(),
        )
        .unwrap(),
    );
    input.items.push(ImplItem::from(counter_method));
    input.into_token_stream().into()
}
