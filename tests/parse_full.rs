use libflate::gzip::Decoder;
use nr_cif::prelude::*;

use std::fs::File;

#[test]
fn test_parse_full() {
    pretty_env_logger::init();

    log::info!("Starting test...");

    let f = File::open("./tests/24-full.cif.gz").expect("cannot read file");
    let cif_result = parse_cif(Decoder::new(f).expect("cannot deflate"));
    match cif_result {
        Ok(file) => {
            log::info!("File parsed. Processing...");
            let mut schedule = ScheduleDatabase::new();
            let errors = schedule.apply_file(&file);
            log::info!("Complete.\n{schedule:#?}\nErrors: {errors:?}");
        }
        Err(e) => panic!("{e}"),
    }
}
