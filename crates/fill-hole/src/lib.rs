use proc_macro::*;

#[proc_macro]
pub fn fill_hole(stream: TokenStream) -> TokenStream {
    let parts = stream.into_iter().collect::<Vec<_>>();
    match &parts[..] {
        [input @ TokenTree::Group(_), TokenTree::Punct(separator), replacement @ TokenTree::Group(_)]
            if separator.to_string() == "," =>
        {
            fill_hole_impl(input, replacement).into()
        }
        _ => panic!("Invalid input: {:?}", parts),
    }
}

fn fill_hole_impl(input: &TokenTree, replacement: &TokenTree) -> TokenTree {
    match input {
        TokenTree::Group(group) => TokenTree::Group(Group::new(
            group.delimiter(),
            group
                .stream()
                .into_iter()
                .map(|element| fill_hole_impl(&element, replacement))
                .collect(),
        )),
        TokenTree::Ident(ident) if ident.to_string() == "_" => replacement.clone(),
        other => other.clone(),
    }
}
