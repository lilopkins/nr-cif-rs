use nr_cif::prelude::*;
use libflate::gzip::Decoder;

use std::fs::File;

#[test]
fn test_parse_full() {
    let f = File::open("./tests/24-full.cif.gz").expect("cannot read file");
    let cif_result = parse_cif(Decoder::new(f).expect("cannot deflate"));
    match cif_result {
        Ok(file) => println!("{file:?}"),
        Err(e) => panic!("{e}"),
    }
}
