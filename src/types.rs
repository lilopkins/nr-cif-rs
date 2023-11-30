use chrono::{NaiveDate, NaiveTime};
use getset::{Getters, MutGetters};

#[derive(Debug, Clone, Getters, MutGetters)]
pub struct CIFFile {
    #[getset(get = "pub", get_mut = "pub(crate)")]
    records: Vec<CIFRecord>,
}

impl CIFFile {
    pub(crate) fn new() -> Self {
        Self { records: vec![] }
    }
}

#[derive(Debug, Clone)]
pub enum CIFRecord {
    Header {
        file_mainframe_identity: String,
        date_of_extract: NaiveDate,
        time_of_extract: NaiveTime,
        current_file_reference: String,
        last_file_reference: String,
        update_indicator: CIFUpdateIndicator,
        version: char,
        user_start_date: NaiveDate,
        user_end_date: NaiveDate,
    },
    TIPLOCInsert {
        tiploc: String,
        capitals_identification: u8,
        nlc: u32,
        nlc_check_char: char,
        tps_description: String,
        stanox: u32,
        three_alpha_code: String,
        nlc_description: String,
    },
    TIPLOCAmend {
        tiploc: String,
        capitals_identification: u8,
        nlc: u32,
        nlc_check_char: char,
        tps_description: String,
        stanox: u32,
        three_alpha_code: String,
        nlc_description: String,
        new_tiploc: String,
    },
    TIPLOCDelete {
        tiploc: String,
    },
    Trailer,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CIFUpdateIndicator {
    /// This is a CIF update ("U")
    #[default]
    Update,
    /// This is a CIF full file ("F")
    Full,
}
