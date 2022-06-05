use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, ImplItem, ImplItemMethod, ItemFn, ItemImpl, Expr, ExprCall, Visibility, Signature, PatType, ReturnType, PatIdent, Pat, Type, TypeImplTrait, TypeParamBound, TraitBound, TraitBoundModifier, Path, PathSegment, PathArguments, ParenthesizedGenericArguments, token, Block, FnArg, ExprPath, Stmt};
use syn::__private::Span;
use syn::__private::Default;
use syn::FnArg::Typed;
use syn::punctuated::Punctuated;

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
        .push(syn::parse(quote!(pub const METHOD_COUNT: usize = #count;).into()).unwrap());
    input.into_token_stream().into()
}

/// Injects a method into the impl block that prints the name of the functions and their place (0-
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

/// Test macro to inject a function that invokes all functions in the impl block (all methods in the
/// annotated block must share the same signature). Note that rather than returning, the
/// invoke_all function will share the same parameter signature as the impl block functions but
/// also has a "consumer" FnMut(Original Return Type) -> () parameter added.
#[proc_macro_attribute]
pub fn invoke_all(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    // Get identifier of the struct type this impl block is on
    let struct_path = if let Type::Path(ref tp) = *input.self_ty {
        tp.path.segments[0].clone()
    } else {
        PathSegment {
            ident: Ident::new("Shouldn't be here!", Span::call_site()),
            arguments: PathArguments::None
        }
    };

    // Get a vec of references to ImplItemMethods in the impl block
    let names = input
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Get output type:
    let output_type = names[0].sig.output.clone();

    // Set up the signature for the invoke_all function.
    let mut invoke_sig = Signature {
        // Set function name to invoke_all
        ident: Ident::new("invoke_all", Span::call_site()),
        // Set return type to ()
        output: ReturnType::Default,
        ..names[0].sig.clone()
    };

    // Grab parameter identifiers to invoke_all before appending consumer closure parameter
    let param_ids = invoke_sig
        .inputs
        .iter()
        .cloned()
        .filter_map(|fnarg| match fnarg {
            FnArg::Receiver(_) => None,
            Typed(pattype) => Some(pattype),
        })
        .filter_map(|pat| match *pat.pat {
            Pat::Ident(patident) => Some(patident.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Specify name of closure parameter:
    let closure_name = "consumer";

    // Use method return type to create an impl trait definition for consumer closures
    invoke_sig.inputs.push(Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: Some(token::Mut {
                span: Span::call_site()
            }),
            // Parameter name to consumer
            ident: Ident::new(closure_name, Span::call_site()),
            subpat: None
        })),
        colon_token: Default::default(),
        // Set parameter type to impl FnMut(IMPL FUNCTIONS RETURN TYPE) -> ()
        ty: Box::new(Type::ImplTrait(TypeImplTrait {
            impl_token: Default::default(),
            bounds: {
                let mut bounds: Punctuated<_, _> = Punctuated::new();
                bounds.push(TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: {
                            let mut segments: Punctuated<_, _> = Punctuated::new();
                            segments.push(PathSegment {
                                ident: Ident::new("FnMut", Span::call_site()),
                                arguments: PathArguments::Parenthesized(ParenthesizedGenericArguments {
                                    paren_token: Default::default(),
                                    inputs: {
                                        let mut inputs: Punctuated<_, _> = Punctuated::new();
                                        // If the methods return anything, FnMut should take it as sole argument
                                        if let ReturnType::Type(_, bx) = output_type {
                                            inputs.push(*bx);
                                        }
                                        inputs
                                    },
                                    output: ReturnType::Default
                                })
                            });
                            segments
                        }
                    }
                }));
                bounds
            }
        }))
    }));

    // By this point, supposing the methods have signatures like pub fn name<T: Trait>(arg: T) -> r
    // The invoke_all function has signature
    // pub fn invoke_all<T: Trait>(arg: T, mut consumer: FnMut(r) -> ()) -> ()

    // Set up body block for the invoke_all method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![]
    };

    // Iterating over names, call consumer to consume a call of a given function:
    for &name in &names {
        // Call function with forwarded parameters
        let inner_call = ExprCall {
            attrs: vec![],
            // Specify function to be called
            func: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: {
                        let mut function_path: Punctuated<_, _> = Punctuated::new();
                        // Add name of struct to path
                        function_path.push(struct_path.clone());
                        // Add name of function to path
                        function_path.push(PathSegment {
                            ident: name.sig.ident.clone(),
                            arguments: PathArguments::None,
                        });
                        function_path
                    }
                }
            })),
            paren_token: Default::default(),
            // forward invoke_all parameters as arguments to this call
            args: {
                let mut args: Punctuated<_, _> = Punctuated::new();
                for identifier in param_ids.iter().cloned() {
                    args.push(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: {
                                let mut p: Punctuated<_, _> = Punctuated::new();
                                p.push(PathSegment {
                                    ident: identifier,
                                    arguments: PathArguments::None,
                                });
                                p
                            }
                        }
                    }))
                }
                args
            }
        };

        // Insert previous call into a call of consumer:
        let outer_call = ExprCall {
            attrs: vec![],
            func: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: {
                        let mut function_path: Punctuated<_, _> = Punctuated::new();
                        function_path.push(PathSegment {
                            ident: Ident::new(closure_name, Span::call_site()),
                            arguments: PathArguments::None,
                        });
                        function_path
                    }
                }
            })),
            paren_token: Default::default(),
            args: {
                let mut args: Punctuated<_, _> = Punctuated::new();
                args.push(Expr::Call(inner_call));
                args
            }
        };

        // Insert combined call into statements
        let ts = outer_call.to_token_stream().to_string();
        invoke_block.stmts.push(Stmt::Semi(Expr::Call(outer_call), Default::default()));
    }

    // Combine invoke_sig and invoke_block into an actual combined function and add to input
    input.items.push(ImplItem::Method(ImplItemMethod {
        attrs: vec![],
        vis: Visibility::Inherited,
        defaultness: None,
        sig: invoke_sig,
        block: invoke_block,
    }));


    input.into_token_stream().into()
}
