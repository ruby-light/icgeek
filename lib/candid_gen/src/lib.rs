use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, Meta, NestedMeta};

struct MethodAttribute {
    canister_name: String,
    method_name: String,
    args_name: String,
    response_name: String,
    candid_method_name: String,
}

#[proc_macro]
pub fn generate_init_candid_method(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AttributeArgs);
    let attribute = get_method_attribute(input);

    let canister_name = format_ident!("{}", attribute.canister_name);
    let method_name = format_ident!("{}", attribute.method_name);
    let args_name = format_ident!("{}", attribute.args_name);

    let args_name = quote! { #canister_name::#method_name::#args_name };

    TokenStream::from(quote! {
        #[candid::candid_method(init)]
        fn init(_: #args_name) {
            unimplemented!();
        }
    })
}

#[proc_macro]
pub fn generate_update_candid_method(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AttributeArgs);
    let attribute = get_method_attribute(input);

    let canister_name = format_ident!("{}", attribute.canister_name);
    let method_name = format_ident!("{}", attribute.method_name);
    let args_name = format_ident!("{}", attribute.args_name);
    let response_name = format_ident!("{}", attribute.response_name);
    let candid_method_name = format_ident!("{}", attribute.candid_method_name);

    let args_name = if args_name == "None" {
        quote! {}
    } else {
        quote! { _: #canister_name::#method_name::#args_name }
    };

    let response_name = if response_name == "Stub" {
        quote! { -> () }
    } else {
        quote! { -> #canister_name::#method_name::#response_name }
    };

    let tokens = quote! {
        #[candid::candid_method(update)]
        fn #candid_method_name(#args_name) #response_name {
            unimplemented!();
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro]
pub fn generate_query_candid_method(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AttributeArgs);
    let attribute = get_method_attribute(input);

    let canister_name = format_ident!("{}", attribute.canister_name);
    let method_name = format_ident!("{}", attribute.method_name);
    let args_name = format_ident!("{}", attribute.args_name);
    let response_name = format_ident!("{}", attribute.response_name);
    let candid_method_name = format_ident!("{}", attribute.candid_method_name);

    let args_name = if args_name == "None" {
        quote! {}
    } else {
        quote! { _: #canister_name::#method_name::#args_name }
    };

    let response_name = if response_name == "Stub" {
        quote! { -> () }
    } else {
        quote! { -> #canister_name::#method_name::#response_name }
    };

    let tokens = quote! {
        #[candid::candid_method(query)]
        fn #candid_method_name(#args_name) #response_name {
            unimplemented!();
        }
    };

    TokenStream::from(tokens)
}

fn get_method_attribute(attrs: AttributeArgs) -> MethodAttribute {
    let canister_name = if let NestedMeta::Meta(Meta::Path(c)) = attrs.get(0).unwrap() {
        c.get_ident().unwrap().to_string()
    } else {
        panic!("Unrecognised 'canister_name' value");
    };

    let method_name = if let NestedMeta::Meta(Meta::Path(m)) = attrs.get(1).unwrap() {
        m.get_ident().unwrap().to_string()
    } else {
        panic!("Unrecognised 'method_name' value");
    };

    let args_name = match attrs.get(2) {
        Some(NestedMeta::Meta(Meta::Path(m))) => m.get_ident().unwrap().to_string(),
        _ => "Args".to_string(),
    };

    let response_name = match attrs.get(3) {
        Some(NestedMeta::Meta(Meta::Path(m))) => m.get_ident().unwrap().to_string(),
        _ => "Response".to_string(),
    };

    let candid_method_name = match attrs.get(4) {
        Some(NestedMeta::Meta(Meta::Path(m))) => m.get_ident().unwrap().to_string(),
        _ => method_name.clone(),
    };

    MethodAttribute {
        canister_name,
        method_name,
        args_name,
        response_name,
        candid_method_name,
    }
}
