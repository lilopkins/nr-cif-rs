use chrono::NaiveDate;
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

            let cancelled_service = "C11004";
            let cancelled_date = NaiveDate::parse_from_str("2024-06-01", "%Y-%m-%d").unwrap();
            let applicable_schedules = schedule
                .schedules()
                .get(cancelled_service)
                .expect("the cancelled service to exist");
            let mut found_cancellation = false;
            for sched in applicable_schedules {
                if *sched.runs_from() == cancelled_date && *sched.runs_to() == cancelled_date {
                    found_cancellation = true;
                    break;
                }
            }
            if !found_cancellation {
                panic!("Cancellation record was missed! {applicable_schedules:?}");
            }

            log::info!("Complete.\n{schedule:#?}\nErrors: {errors:?}");
        }
        Err(e) => panic!("{e}"),
    }
}
