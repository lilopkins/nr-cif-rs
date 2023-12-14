use fixedlength_format_parser::FixedLengthFormatParser;
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

#[derive(Debug, Clone, FixedLengthFormatParser)]
pub enum CIFRecord {
    #[record_type = "HD"]
    Header {
        #[field_starts = 2]
        #[field_length = 20]
        file_mainframe_identity: String,
        #[field_length = 6]
        date_of_extract: String,
        #[field_length = 4]
        time_of_extract: String,
        #[field_length = 7]
        current_file_reference: String,
        #[field_length = 7]
        last_file_reference: String,
        #[field_length = 1]
        update_indicator: char,
        #[field_length = 1]
        version: char,
        #[field_length = 6]
        user_start_date: String,
        #[field_length = 6]
        user_end_date: String,
    },

    #[record_type = "TI"]
    TIPLOCInsert {
        #[field_starts = 2]
        #[field_length = 7]
        tiploc: String,
        #[field_length = 2]
        capitals_identification: u8,
        #[field_length = 6]
        nlc: u32,
        #[field_length = 1]
        nlc_check_char: char,
        #[field_length = 26]
        tps_description: String,
        #[field_length = 5]
        stanox: u32,
        #[field_length = 4]
        po_mcp_code: String,
        #[field_length = 3]
        three_alpha_code: String,
        #[field_length = 16]
        nlc_description: String,
    },

    #[record_type = "TA"]
    TIPLOCAmend {
        #[field_starts = 2]
        #[field_length = 7]
        tiploc: String,
        #[field_length = 2]
        capitals_identification: u8,
        #[field_length = 6]
        nlc: u32,
        #[field_length = 1]
        nlc_check_char: char,
        #[field_length = 26]
        tps_description: String,
        #[field_length = 5]
        stanox: u32,
        #[field_length = 4]
        po_mcp_code: String,
        #[field_length = 3]
        three_alpha_code: String,
        #[field_length = 16]
        nlc_description: String,
        #[field_length = 7]
        new_tiploc: String,
    },

    #[record_type = "TD"]
    TIPLOCDelete {
        #[field_starts = 2]
        #[field_length = 7]
        tiploc: String,
    },

    #[record_type = "AA"]
    Association {
        #[field_starts = 2]
        #[field_length = 1]
        transaction_type: char,
        #[field_length = 6]
        main_train_uid: String,
        #[field_length = 6]
        associated_train_uid: String,
        #[field_length = 6]
        association_start_date: String,
        #[field_length = 6]
        association_end_date: String,
        #[field_length = 7]
        association_days: String,
        #[field_length = 2]
        association_category: String,
        #[field_length = 1]
        association_date_indicator: char,
        #[field_length = 7]
        association_location: String,
        #[field_length = 1]
        base_location_suffix: String,
        #[field_length = 1]
        association_location_suffix: String,
        #[field_length = 1]
        diagram_type: char,
        #[field_length = 1]
        association_type: char,
        #[field_starts = 79]
        #[field_length = 1]
        stp_indicator: char,
    },

    #[record_type = "BS"]
    BasicSchedule {
        #[field_starts = 2]
        #[field_length = 1]
        transaction_type: char,
        #[field_length = 6]
        train_uid: String,
        #[field_length = 6]
        date_runs_from: String,
        #[field_length = 6]
        date_runs_to: String,
        #[field_length = 7]
        days_run: String,
        #[field_length = 1]
        bank_holiday_running: char,
        #[field_length = 1]
        train_status: char,
        #[field_length = 2]
        train_category: String,
        #[field_length = 4]
        train_identity: String,
        #[field_length = 4]
        headcode: String,
        #[field_length = 1]
        course_indicator: char,
        #[field_length = 8]
        train_service_code: String,
        #[field_length = 1]
        portion_id: char,
        #[field_length = 3]
        power_type: String,
        #[field_length = 4]
        timing_load: String,
        #[field_length = 3]
        speed: String,
        #[field_length = 6]
        operating_characteristics: String,
        #[field_length = 1]
        seating_class: char,
        #[field_length = 1]
        sleepers: char,
        #[field_length = 1]
        reservations: char,
        #[field_length = 1]
        connection_indicator: char,
        #[field_length = 4]
        catering_code: String,
        #[field_length = 4]
        service_branding: String,
        #[field_starts = 79]
        #[field_length = 1]
        stp_indicator: char,
    },

    #[record_type = "BX"]
    BasicScheduleExtended {
        #[field_starts = 2]
        #[field_length = 4]
        traction_class: String,
        #[field_length = 5]
        uic_code: String,
        #[field_length = 2]
        atoc_code: String,
        #[field_length = 1]
        applicable_timetable_code: char,
    },

    #[record_type = "LO"]
    LocationOrigin {
        #[field_starts = 2]
        #[field_length = 8]
        location: String,
        #[field_length = 5]
        scheduled_departure_time: String,
        #[field_length = 4]
        public_departure_time: String,
        #[field_length = 3]
        platform: String,
        #[field_length = 3]
        line: String,
        #[field_length = 2]
        engineering_allowance: String,
        #[field_length = 2]
        pathing_allowance: String,
        #[field_length = 12]
        activity: String,
        #[field_length = 2]
        performance_allowance: String,
    },

    #[record_type = "LI"]
    LocationIntermediate {
        #[field_starts = 2]
        #[field_length = 8]
        location: String,
        #[field_length = 5]
        scheduled_arrival_time: String,
        #[field_length = 5]
        scheduled_departure_time: String,
        #[field_length = 5]
        scheduled_pass: String,
        #[field_length = 4]
        public_arrival_time: String,
        #[field_length = 4]
        public_departure_time: String,
        #[field_length = 3]
        platform: String,
        #[field_length = 3]
        line: String,
        #[field_length = 3]
        path: String,
        #[field_length = 12]
        activity: String,
        #[field_length = 2]
        engineering_allowance: String,
        #[field_length = 2]
        pathing_allowance: String,
        #[field_length = 2]
        performance_allowance: String,
    },

    #[record_type = "CR"]
    ChangeEnRoute {
        #[field_starts = 2]
        #[field_length = 8]
        location: String,
        #[field_length = 2]
        train_category: String,
        #[field_length = 4]
        train_identity: String,
        #[field_length = 4]
        headcode: String,
        #[field_length = 1]
        course_indicator: char,
        #[field_length = 8]
        profit_centre_code: String,
        #[field_length = 1]
        business_sector: char,
        #[field_length = 3]
        power_type: String,
        #[field_length = 4]
        timing_load: String,
        #[field_length = 3]
        speed: String,
        #[field_length = 6]
        operating_chars: String,
        #[field_length = 1]
        train_class: char,
        #[field_length = 1]
        sleepers: char,
        #[field_length = 1]
        reservations: char,
        #[field_length = 1]
        connect_indicator: char,
        #[field_length = 4]
        catering_code: String,
        #[field_length = 4]
        service_branding: String,
        #[field_length = 4]
        traction_class: String,
        #[field_length = 5]
        uic_code: String,
        #[field_length = 8]
        retail_train_id: String,
    },

    #[record_type = "LT"]
    LocationTerminate {
        #[field_starts = 2]
        #[field_length = 8]
        location: String,
        #[field_length = 5]
        scheduled_arrival_time: String,
        #[field_length = 4]
        public_arrival_time: String,
        #[field_length = 3]
        platform: String,
        #[field_length = 3]
        path: String,
        #[field_length = 12]
        activity: String,
    },

    #[record_type = "ZZ"]
    Trailer,
}
