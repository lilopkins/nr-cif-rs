use nr_cif::prelude::*;

use std::fs::File;

#[test]
fn test_parse_full() {
    let f = File::open("./tests/24-full.cif").expect("cannot read file");
    let cif_result = parse_cif(f);
    match cif_result {
        Ok(file) => panic!("{file:?}"),
        Err(e) => panic!("{e}"),
    }
}
