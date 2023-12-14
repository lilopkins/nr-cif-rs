use chrono::NaiveDate;
use libflate::gzip::Decoder;
use nr_cif::prelude::*;
use ron::ser::PrettyConfig;

use std::{
    env,
    fs::{self, File},
};

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

            // Test cancelled services are registered
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

            // Test TIPLOCs don't have trailing number (issue #9)
            for (_, sched_stack) in schedule.schedules() {
                for sched in sched_stack {
                    for waypoint in sched.journey() {
                        assert!(waypoint.tiploc().len() <= 7, "TIPLOC too long");
                    }
                }
            }

            // Test getting CRS from STANOX (issue #8)
            assert!(
                schedule
                    .get_crs_from_tiploc("WATRLMN")
                    .contains(&"WAT".to_string()),
                "WATRLMN should have CRS 'WAT'."
            );
            assert!(
                schedule
                    .get_crs_from_tiploc("CLPHMJ1")
                    .contains(&"CLJ".to_string()),
                "CLPHMJ1 should have CRS 'CLJ'."
            );
            assert!(
                schedule
                    .get_crs_from_tiploc("CLPHMJW")
                    .contains(&"CLJ".to_string()),
                "CLPHMJW should have CRS 'CLJ'."
            );
            assert!(
                schedule
                    .get_crs_from_tiploc("VAUXHLM")
                    .contains(&"VXH".to_string()),
                "WREXGUB should have CRS 'WRX'."
            );
            assert!(
                schedule
                    .get_crs_from_tiploc("WEYBDGB")
                    .contains(&"WYB".to_string()),
                "WEYBDGB should have CRS 'WYB'."
            );

            log::info!("Complete.\nErrors: {errors:?}");
            if env::var("SAVE_PARSED_OUTPUT")
                .unwrap_or("no".to_string())
                .to_ascii_lowercase()
                == "yes"
            {
                let path = "./target/test_parsed_schedule.ron";
                log::info!("Saving output to {path}.");
                let f = fs::File::create(path).expect("Should be able to write file.");
                ron::ser::to_writer_pretty(f, &schedule, PrettyConfig::default())
                    .expect("Should be able to write output.");
            } else {
                log::info!("Not saving output.");
            }
        }
        Err(e) => panic!("{e}"),
    }
}
