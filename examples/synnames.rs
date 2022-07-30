fn main() {
    let set = syntect::parsing::SyntaxSet::load_defaults_newlines();
    for syn in set.syntaxes() {
        println!("{}: {:?}", &syn.name, syn.file_extensions);
    }
}