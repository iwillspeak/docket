use pulldown_cmark::*;
use std::{fs::File, io::Read};

fn main() {
    for arg in std::env::args().skip(1) {
        let mut file = File::open(arg).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let parser = Parser::new_ext(&contents, Options::all());
        for event in parser {
            println!("{event:?}");
        }
    }
}
