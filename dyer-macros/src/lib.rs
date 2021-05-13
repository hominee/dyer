extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn middleware(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !item.to_string().trim().starts_with("pub") {
        let s = "pub ".to_string() + &item.to_string();
        item= TokenStream::from_str(&s).unwrap();
        println!("item: {}", item.to_string());
    }
    item
}

#[proc_macro_attribute]
pub fn pipeline(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !item.to_string().trim().starts_with("pub") {
        let s = "pub ".to_string() + &item.to_string();
        item= TokenStream::from_str(&s).unwrap();
    }
    item
}

#[proc_macro_attribute]
pub fn spider(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !item.to_string().trim().starts_with("pub") {
        let s = "pub ".to_string() + &item.to_string();
        item= TokenStream::from_str(&s).unwrap();
    }
    item
}

#[proc_macro_attribute]
pub fn parser(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !item.to_string().trim().starts_with("pub") {
        let s = "pub ".to_string() + &item.to_string();
        item= TokenStream::from_str(&s).unwrap();
    }
    item
}
