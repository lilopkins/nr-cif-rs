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

impl CIFRecord {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CIFRecord::Header {
                file_mainframe_identity: _,
                date_of_extract: _,
                time_of_extract: _,
                current_file_reference: _,
                last_file_reference: _,
                update_indicator: _,
                version: _,
                user_start_date: _,
                user_end_date: _,
            } => "HD",
            CIFRecord::TIPLOCInsert {
                tiploc: _,
                capitals_identification: _,
                nlc: _,
                nlc_check_char: _,
                tps_description: _,
                stanox: _,
                po_mcp_code: _,
                three_alpha_code: _,
                nlc_description: _,
            } => "TI",
            CIFRecord::TIPLOCAmend {
                tiploc: _,
                capitals_identification: _,
                nlc: _,
                nlc_check_char: _,
                tps_description: _,
                stanox: _,
                po_mcp_code: _,
                three_alpha_code: _,
                nlc_description: _,
                new_tiploc: _,
            } => "TA",
            CIFRecord::TIPLOCDelete { tiploc: _ } => "TD",
            CIFRecord::Association {
                transaction_type: _,
                main_train_uid: _,
                associated_train_uid: _,
                association_start_date: _,
                association_end_date: _,
                association_days: _,
                association_category: _,
                association_date_indicator: _,
                association_location: _,
                base_location_suffix: _,
                association_location_suffix: _,
                diagram_type: _,
                association_type: _,
                stp_indicator: _,
            } => "AA",
            CIFRecord::BasicSchedule {
                transaction_type: _,
                train_uid: _,
                date_runs_from: _,
                date_runs_to: _,
                days_run: _,
                bank_holiday_running: _,
                train_status: _,
                train_category: _,
                train_identity: _,
                headcode: _,
                course_indicator: _,
                train_service_code: _,
                portion_id: _,
                power_type: _,
                timing_load: _,
                speed: _,
                operating_characteristics: _,
                seating_class: _,
                sleepers: _,
                reservations: _,
                connection_indicator: _,
                catering_code: _,
                service_branding: _,
                stp_indicator: _,
            } => "BS",
            CIFRecord::BasicScheduleExtended {
                traction_class: _,
                uic_code: _,
                atoc_code: _,
                applicable_timetable_code: _,
            } => "BX",
            CIFRecord::LocationOrigin {
                location: _,
                scheduled_departure_time: _,
                public_departure_time: _,
                platform: _,
                line: _,
                engineering_allowance: _,
                pathing_allowance: _,
                activity: _,
                performance_allowance: _,
            } => "LO",
            CIFRecord::LocationIntermediate {
                location: _,
                scheduled_arrival_time: _,
                scheduled_departure_time: _,
                scheduled_pass: _,
                public_arrival_time: _,
                public_departure_time: _,
                platform: _,
                line: _,
                path: _,
                activity: _,
                engineering_allowance: _,
                pathing_allowance: _,
                performance_allowance: _,
            } => "LI",
            CIFRecord::ChangeEnRoute {
                location: _,
                train_category: _,
                train_identity: _,
                headcode: _,
                course_indicator: _,
                profit_centre_code: _,
                business_sector: _,
                power_type: _,
                timing_load: _,
                speed: _,
                operating_chars: _,
                train_class: _,
                sleepers: _,
                reservations: _,
                connect_indicator: _,
                catering_code: _,
                service_branding: _,
                traction_class: _,
                uic_code: _,
                retail_train_id: _,
            } => "CR",
            CIFRecord::LocationTerminate {
                location: _,
                scheduled_arrival_time: _,
                public_arrival_time: _,
                platform: _,
                path: _,
                activity: _,
            } => "LT",
            CIFRecord::Trailer => "ZZ",
        }
    }
}
