use quote::quote;

fn main() {
    let empty = quote! {};
    println!("Empty quote: {}", empty);

    let with_content = quote! {
        fn foo() {}
    };
    println!("With content: {}", with_content);
}
