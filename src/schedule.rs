use std::{collections::HashMap, str::FromStr};

use bitflags::bitflags;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use getset::Getters;
use log::{info, trace, warn};
use thiserror::Error;

use crate::types::{CIFFile, CIFRecord};

#[derive(Error, Debug)]
pub enum ScheduleApplyError {
    #[error("invalid extract date and time in header record")]
    InvalidHeaderDateTime(String),
    #[error("invalid date in basic schedule record")]
    InvalidScheduleDate(String),
    #[error("invalid days run in basic schedule record")]
    InvalidDaysRun(String),
    #[error("invalid train status in basic schedule record")]
    InvalidTrainStatus(char),
    #[error("invalid train category in basic schedule record")]
    InvalidTrainCategory(String),
    #[error("invalid train power type in basic schedule record")]
    InvalidPowerType(String),
    #[error("invalid timing load in basic schedule record")]
    InvalidTimingLoad(String),
    #[error("invalid operating characteristic in basic schedule record")]
    InvalidOperatingCharacteristic(char),
    #[error("invalid seating class in basic schedule record")]
    InvalidSeatingClass(char),
    #[error("invalid sleepers value in basic schedule record")]
    InvalidSleepers(char),
    #[error("invalid reservations in basic schedule record")]
    InvalidReservations(char),
    #[error("invalid catering code in basic schedule record")]
    InvalidCateringCode(char),
    #[error("invalid STP indicator in basic schedule record")]
    InvalidSTPIndicator(char),
    #[error("invalid journey time in location record")]
    InvalidJourneyTime(String),
}

#[derive(Debug, Clone, Getters)]
pub struct ScheduleDatabase {
    #[getset(get = "pub")]
    extract_date_time: NaiveDateTime,
    #[getset(get = "pub")]
    tiplocs: HashMap<String, TIPLOC>,
    #[getset(get = "pub")]
    schedules: HashMap<String, Vec<Schedule>>,
}

impl Default for ScheduleDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleDatabase {
    /// Create a new [`ScheduleDatabase`].
    pub fn new() -> Self {
        Self {
            extract_date_time: NaiveDateTime::MIN,
            tiplocs: HashMap::new(),
            schedules: HashMap::new(),
        }
    }

    /// Apply a file onto this schedule database.
    /// This can reset the database if this includes a full update.
    pub fn apply_file(&mut self, file: &CIFFile) -> Vec<(usize, ScheduleApplyError)> {
        self.apply_records(file.records())
    }

    /// Apply a list of records onto this schedule database.
    /// This can reset the database if this includes a full update.
    /// Returns a list of errors and their respective record index.
    pub fn apply_records(&mut self, records: &[CIFRecord]) -> Vec<(usize, ScheduleApplyError)> {
        let mut bundle = vec![];
        let mut errors = vec![];
        for (record_idx, record) in records.iter().enumerate() {
            if record_idx % 10000 == 0 {
                info!(
                    "Processing record #{}. (shown every 10000 records)",
                    record_idx + 1
                );
            }
            bundle.push(record);

            // Check type
            let submit = match record.kind() {
                "HD" => true,
                "ZZ" => true,
                "AA" => true,
                "LT" => true,
                "TI" => true,
                "TA" => true,
                "TD" => true,
                "BS" => {
                    if let CIFRecord::BasicSchedule {
                        transaction_type,
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
                        stp_indicator,
                    } = record
                    {
                        // only submit a BS record alone if it's a delete, or a cancellation
                        *transaction_type == 'D' || *stp_indicator == 'C'
                    } else {
                        false
                    }
                }
                _ => false,
            };
            trace!("Record type: {}, submitting: {submit}", record.kind());

            if submit {
                let r = if bundle.len() == 1 {
                    self.apply_single_record(bundle[0])
                } else {
                    self.apply_record_bundle(&bundle)
                };
                if let Err(e) = r {
                    #[cfg(feature = "panic-on-first-error")]
                    {
                        log::error!("Error at record {record_idx}, line {}", record_idx + 1);
                        log::error!("Error: {e:?}");
                        log::error!("Records: {:?}", bundle);
                    }
                    errors.push((record_idx, e));
                    #[cfg(feature = "panic-on-first-error")]
                    panic!(
                        "Came across an error and the `panic-on-first-error` feature is enabled."
                    );
                }
                bundle.clear();
            }
        }
        errors
    }

    /// Apply a single record. A bundle consists of either:
    /// - A header record.
    /// - A TIPLOC record.
    /// - An association record.
    /// - A lone BS record for a delete type.
    /// - A trailer record.
    fn apply_single_record(&mut self, record: &CIFRecord) -> Result<(), ScheduleApplyError> {
        match record {
            CIFRecord::Header {
                file_mainframe_identity: _,
                date_of_extract,
                time_of_extract,
                current_file_reference: _,
                last_file_reference: _,
                update_indicator,
                version: _,
                user_start_date: _,
                user_end_date: _,
            } => {
                if *update_indicator == 'F' {
                    // full update, empty database
                    info!("Received full update, clearing database.");
                    self.tiplocs.clear();
                }
                let date = NaiveDate::parse_from_str(date_of_extract, "%d%m%y").map_err(|_| {
                    ScheduleApplyError::InvalidHeaderDateTime(date_of_extract.clone())
                })?;
                let time = NaiveTime::parse_from_str(time_of_extract, "%H%M").map_err(|_| {
                    ScheduleApplyError::InvalidHeaderDateTime(time_of_extract.clone())
                })?;
                self.extract_date_time = date.and_time(time);
            }

            CIFRecord::TIPLOCInsert {
                tiploc,
                capitals_identification: _,
                nlc: _,
                nlc_check_char: _,
                tps_description,
                stanox: _,
                po_mcp_code: _,
                three_alpha_code,
                nlc_description: _,
            } => {
                info!("New TIPLOC: {}", tiploc.trim());
                self.tiplocs.insert(
                    tiploc.trim().to_string(),
                    TIPLOC {
                        tiploc: tiploc.trim().to_string(),
                        three_alpha_code: three_alpha_code.trim().to_string(),
                        description: tps_description.trim().to_string(),
                    },
                );
            }
            CIFRecord::TIPLOCAmend {
                tiploc,
                capitals_identification: _,
                nlc: _,
                nlc_check_char: _,
                tps_description,
                stanox: _,
                po_mcp_code: _,
                three_alpha_code,
                nlc_description: _,
                new_tiploc,
            } => {
                info!("Amendment for TIPLOC {}", tiploc.trim());
                let tiploc = if new_tiploc.trim().is_empty() {
                    tiploc.trim().to_string()
                } else {
                    self.tiplocs.remove(&tiploc.trim().to_string());
                    new_tiploc.trim().to_string()
                };
                self.tiplocs.insert(
                    tiploc.clone(),
                    TIPLOC {
                        tiploc,
                        three_alpha_code: three_alpha_code.trim().to_string(),
                        description: tps_description.trim().to_string(),
                    },
                );
            }
            CIFRecord::TIPLOCDelete { tiploc } => {
                info!("Removed TIPLOC {}", tiploc.trim());
                self.tiplocs.remove(tiploc.trim());
            }

            CIFRecord::BasicSchedule {
                transaction_type,
                train_uid,
                date_runs_from,
                date_runs_to,
                days_run,
                bank_holiday_running,
                train_status,
                train_category,
                train_identity,
                headcode: _,
                course_indicator: _,
                train_service_code: _,
                portion_id,
                power_type,
                timing_load,
                speed,
                operating_characteristics,
                seating_class,
                sleepers,
                reservations,
                connection_indicator: _,
                catering_code,
                service_branding: _,
                stp_indicator,
            } => {
                assert!(
                    *transaction_type == 'D' || *stp_indicator == 'C',
                    "transaction type must be delete, or it must be a cancellation to be processed as a single record"
                );
                if *transaction_type == 'D' {
                    self.schedules.remove(train_uid);
                } else {
                    let mut sch = Schedule::new();
                    bs_record_to_schedule(
                        &mut sch,
                        train_uid,
                        date_runs_from,
                        date_runs_to,
                        days_run,
                        bank_holiday_running,
                        train_status,
                        train_category,
                        train_identity,
                        portion_id,
                        power_type,
                        timing_load,
                        speed,
                        operating_characteristics,
                        seating_class,
                        sleepers,
                        reservations,
                        catering_code,
                        stp_indicator,
                    )?;
                    self.schedules
                        .entry(train_uid.clone())
                        .and_modify(|v| v.push(sch));
                }
            }

            _ => (),
        }
        Ok(())
    }

    /// Apply a bundle of records. A bundle consists of either:
    /// - A schedule, from the BS record to the LT record, for a new or revise types.
    fn apply_record_bundle(
        &mut self,
        record_bundle: &Vec<&CIFRecord>,
    ) -> Result<(), ScheduleApplyError> {
        let mut schedule = Schedule::new();

        for record in record_bundle {
            match record {
                CIFRecord::BasicSchedule {
                    transaction_type,
                    train_uid,
                    date_runs_from,
                    date_runs_to,
                    days_run,
                    bank_holiday_running,
                    train_status,
                    train_category,
                    train_identity,
                    headcode: _,
                    course_indicator: _,
                    train_service_code: _,
                    portion_id,
                    power_type,
                    timing_load,
                    speed,
                    operating_characteristics,
                    seating_class,
                    sleepers,
                    reservations,
                    connection_indicator: _,
                    catering_code,
                    service_branding: _,
                    stp_indicator,
                } => {
                    let uid = train_uid.trim().to_string();
                    if *transaction_type == 'R' && !self.schedules.contains_key(&uid) {
                        warn!("A record is trying to revise schedule {uid}, but it doesn't exist in the database. Inserting it as new...");
                    }

                    bs_record_to_schedule(
                        &mut schedule,
                        &uid,
                        date_runs_from,
                        date_runs_to,
                        days_run,
                        bank_holiday_running,
                        train_status,
                        train_category,
                        train_identity,
                        portion_id,
                        power_type,
                        timing_load,
                        speed,
                        operating_characteristics,
                        seating_class,
                        sleepers,
                        reservations,
                        catering_code,
                        stp_indicator,
                    )?;
                }
                CIFRecord::BasicScheduleExtended {
                    traction_class: _,
                    uic_code: _,
                    atoc_code,
                    applicable_timetable_code,
                } => {
                    schedule.atoc_code = atoc_code.trim().to_string();
                    schedule.subject_to_performance_monitoring = *applicable_timetable_code == 'Y';
                }
                CIFRecord::LocationOrigin {
                    location,
                    scheduled_departure_time,
                    public_departure_time,
                    platform,
                    line,
                    engineering_allowance: _,
                    pathing_allowance: _,
                    activity,
                    performance_allowance: _,
                } => schedule.journey.push(JourneyLocation {
                    tiploc: location.trim().to_string(),
                    arrival_time: None,
                    departure_time: Some(scheduled_departure_time.parse()?),
                    passing_time: None,
                    public_arrival: None,
                    public_departure: Some(public_departure_time.parse()?),
                    platform: platform.trim().to_string(),
                    line: line.trim().to_string(),
                    activity: activity.trim().to_string(),
                }),
                CIFRecord::LocationIntermediate {
                    location,
                    scheduled_arrival_time,
                    scheduled_departure_time,
                    scheduled_pass,
                    public_arrival_time,
                    public_departure_time,
                    platform,
                    line,
                    path: _,
                    activity,
                    engineering_allowance: _,
                    pathing_allowance: _,
                    performance_allowance: _,
                } => schedule.journey.push(JourneyLocation {
                    tiploc: location.trim().to_string(),
                    arrival_time: if scheduled_arrival_time.trim().is_empty() {
                        None
                    } else {
                        Some(scheduled_arrival_time.parse()?)
                    },
                    departure_time: if scheduled_departure_time.trim().is_empty() {
                        None
                    } else {
                        Some(scheduled_departure_time.parse()?)
                    },
                    passing_time: if scheduled_pass.trim().is_empty() {
                        None
                    } else {
                        Some(scheduled_pass.parse()?)
                    },
                    public_arrival: if public_arrival_time.trim().is_empty() {
                        None
                    } else {
                        Some(public_arrival_time.parse()?)
                    },
                    public_departure: if public_departure_time.trim().is_empty() {
                        None
                    } else {
                        Some(public_departure_time.parse()?)
                    },
                    platform: platform.trim().to_string(),
                    line: line.trim().to_string(),
                    activity: activity.trim().to_string(),
                }),
                CIFRecord::LocationTerminate {
                    location,
                    scheduled_arrival_time,
                    public_arrival_time,
                    platform,
                    path: _,
                    activity,
                } => schedule.journey.push(JourneyLocation {
                    tiploc: location.trim().to_string(),
                    arrival_time: Some(scheduled_arrival_time.parse()?),
                    departure_time: None,
                    passing_time: None,
                    public_arrival: Some(public_arrival_time.parse()?),
                    public_departure: None,
                    platform: platform.trim().to_string(),
                    line: String::new(),
                    activity: activity.trim().to_string(),
                }),

                _ => (),
            }
        }

        self.schedules
            .entry(schedule.train_uid.clone())
            .and_modify(|v| v.push(schedule))
            .or_insert(vec![]);
        Ok(())
    }
}

#[derive(Debug, Clone, Getters)]
pub struct TIPLOC {
    /// The TIPLOC code of this location.
    #[getset(get = "pub")]
    tiploc: String,
    /// A 3 letter CRS code, if one is present for this location, or an empty string.
    #[getset(get = "pub")]
    three_alpha_code: String,
    /// The description of this location.
    #[getset(get = "pub")]
    description: String,
}

#[derive(Debug, Clone, Getters)]
pub struct Schedule {
    /// The service identifier.
    #[getset(get = "pub")]
    train_uid: String,
    /// When does this service start running.
    #[getset(get = "pub")]
    runs_from: NaiveDate,
    /// When does this service stop running.
    #[getset(get = "pub")]
    runs_to: NaiveDate,
    /// Days in which this service operates. A bitflag.
    #[getset(get = "pub")]
    days_run: DaysRun,
    /// Details about bank holiday running.
    #[getset(get = "pub")]
    bank_holiday_running: BankHolidayRunning,
    /// Train operating company code.
    #[getset(get = "pub")]
    atoc_code: String,
    /// Is this train subject to performance monitoring.
    #[getset(get = "pub")]
    subject_to_performance_monitoring: bool,
    #[getset(get = "pub")]
    train_status: TrainStatus,
    #[getset(get = "pub")]
    train_category: TrainCategory,
    #[getset(get = "pub")]
    headcode: String,
    #[getset(get = "pub")]
    portion_id: char,
    #[getset(get = "pub")]
    power_type: PowerType,
    #[getset(get = "pub")]
    timing_load: TimingLoad,
    #[getset(get = "pub")]
    speed: u32,
    #[getset(get = "pub")]
    operating_characteristics: Vec<OperatingCharacteristic>,
    #[getset(get = "pub")]
    seating_class: SeatingClass,
    #[getset(get = "pub")]
    sleepers: Sleepers,
    #[getset(get = "pub")]
    reservations: Reservations,
    #[getset(get = "pub")]
    catering: Vec<Catering>,
    #[getset(get = "pub")]
    stp_indicator: STPIndicator,
    #[getset(get = "pub")]
    journey: Vec<JourneyLocation>,
}

impl Schedule {
    fn new() -> Self {
        Self {
            train_uid: String::new(),
            runs_from: NaiveDate::MIN,
            runs_to: NaiveDate::MIN,
            days_run: DaysRun::empty(),
            bank_holiday_running: BankHolidayRunning::RunsNormally,
            atoc_code: String::new(),
            subject_to_performance_monitoring: false,
            train_status: TrainStatus::PassengerAndParcels,
            train_category: TrainCategory::NotSpecified,
            headcode: String::new(),
            portion_id: ' ',
            power_type: PowerType::Diesel,
            timing_load: TimingLoad::LoadInTonnes(0),
            speed: 0,
            operating_characteristics: vec![],
            seating_class: SeatingClass::NotSpecified,
            sleepers: Sleepers::NotSpecified,
            reservations: Reservations::Possible,
            catering: vec![],
            stp_indicator: STPIndicator::PermanentAssociation,
            journey: vec![],
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DaysRun: u8 {
        const MONDAY    = 0b1000000;
        const TUESDAY   = 0b0100000;
        const WEDNESDAY = 0b0010000;
        const THURSDAY  = 0b0001000;
        const FRIDAY    = 0b0000100;
        const SATURDAY  = 0b0000010;
        const SUNDAY    = 0b0000001;

        const WEEKDAYS = Self::MONDAY.bits() | Self::TUESDAY.bits() | Self::WEDNESDAY.bits() | Self::THURSDAY.bits() | Self::FRIDAY.bits();
        const WEEKENDS = Self::SATURDAY.bits() | Self::SUNDAY.bits();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankHolidayRunning {
    RunsNormally,
    NotOnSpecificBankHolidayMondays,
    NotOnGlasgowBankHolidays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainStatus {
    Bus,
    Freight,
    PassengerAndParcels,
    Ship,
    Trip,
    STPPassengerAndParcels,
    STPFreight,
    STPTrip,
    STPShip,
    STPBus,
    NotSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainCategory {
    NotSpecified,
    LondonUnderground,
    UnadvertisedOrdinaryPassenger,
    OrdinaryPassenger,
    StaffTrain,
    Mixed,
    ChannelTunnel,
    Sleeper,
    International,
    Motorail,
    UnadvertisedExpress,
    ExpressPassenger,
    SleeperDomestic,
    BusReplacementDueToEngineering,
    BusWTTService,
    Ship,
    EmptyCoachingStock,
    ECSLondonUnderground,
    ECSAndStaff,
    Postal,
    PostOfficeControlledParcels,
    Parcels,
    EmptyNPCCS,
    Departmental,
    CivilEngineer,
    MechanicalAndElectricalEngineer,
    Stores,
    Test,
    SignalAndTelecommunicationsEngineer,
    LocomotiveAndBrakeVan,
    LightLocomotive,
    RfDAutomotiveComponents,
    RfDAutomotiveVehicles,
    RfDEdibleProducts,
    RfDIndustrialMinerals,
    RfDChemicals,
    RfDBuildingMaterials,
    RfDGeneralMerchandise,
    RfDEuropean,
    RfDFreightlinerContracts,
    RfDFreightlinerOther,
    CoalDistributive,
    CoalElectricityMGR,
    CoalOtherAndNuclear,
    Metals,
    Aggregates,
    DomesticAndIndustrialWaste,
    BuildingMaterials,
    PetroleumProducts,
    RfDEuropeanChannelTunnelMixed,
    RfDEuropeanChannelTunnelIntermodal,
    RfDEuropeanChannelTunnelAutomotive,
    RfDEuropeanChannelTunnelContractServices,
    RfDEuropeanChannelTunnelHaulmark,
    RfDEuropeanChannelTunnelJointVenture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerType {
    Diesel,
    DieselElectricMultipleUnit,
    DieselMechanicalMultipleUnit,
    Electric,
    ElectroDiesel,
    EMUPlusLocomotive,
    ElectricMultipleUnit,
    HighSpeedTrain,
    NotSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingCharacteristic {
    VacuumBraked,
    TimedAt100MPH,
    DOOCoachingStockTrains,
    ConveysMark4Coaches,
    GuardRequired,
    TimedAt110MPH,
    PushPullTrain,
    RunsAsRequired,
    AirConditionedWithPASystem,
    SteamHeated,
    RunsToTerminalsAsRequired,
    MayConveyTrafficToSB1CGauge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingLoad {
    /// Unspecifed
    NotSpecified,
    /// Class 170/0, 172/1, 172/2
    Class17201721Or1722,
    /// Class 141 to 144
    Class141To144,
    /// Class 158, 168, 170 or 175
    Class158168170Or175,
    /// Class 165/0
    Class1650,
    /// Class 150, 153, 155 or 156
    Class150153155Or156,
    /// Class 165/1 or 166
    Class1651Or166,
    /// Class 220 or 221
    Class220Or221,
    /// Class 159
    Class159,
    /// DMU (Power Car + Trailer)
    DMUPowerCarTrailer,
    /// DMU (2 Power Cars + Trailer)
    DMU2PowerCarsTrailer,
    /// DMU (Power Twin)
    DMUPowerTwin,
    /// Accelerated Timings EMU
    AcceleratedTimings,
    /// Class 458
    Class458,
    /// Class 380
    Class380,
    /// Class 350/1 (110 mph)
    Class3501110MPH,
    /// Class 325 Electric Parcels Unit
    Class325ElectricParcelsUnit,
    /// Specific class
    SpecificClass(u16),
    /// Load in tonnes
    LoadInTonnes(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatingClass {
    FirstAndStandard,
    StandardOnly,
    NotSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sleepers {
    FirstAndStandard,
    FirstOnly,
    StandardOnly,
    NotSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reservations {
    Compulsory,
    CompulsoryForBicycles,
    Recommended,
    Possible,
    NotSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Catering {
    NotSpecified,
    BuffetService,
    RestaurantCarForFirstClass,
    HotFood,
    MealForFirstClass,
    WheelchairReservations,
    Restaurant,
    TrolleyService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum STPIndicator {
    NewSTPAssociation,
    STPCancellationOfPermanentAssociation,
    STPOverlayOfPermanentAssociation,
    PermanentAssociation,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct JourneyLocation {
    #[getset(get = "pub")]
    tiploc: String,
    #[getset(get = "pub")]
    arrival_time: Option<JourneyTime>,
    #[getset(get = "pub")]
    departure_time: Option<JourneyTime>,
    #[getset(get = "pub")]
    passing_time: Option<JourneyTime>,
    #[getset(get = "pub")]
    public_arrival: Option<JourneyTime>,
    #[getset(get = "pub")]
    public_departure: Option<JourneyTime>,
    #[getset(get = "pub")]
    platform: String,
    #[getset(get = "pub")]
    line: String,
    #[getset(get = "pub")]
    activity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct JourneyTime {
    #[getset(get = "pub")]
    hour: u8,
    #[getset(get = "pub")]
    minute: u8,
    #[getset(get = "pub")]
    half: bool,
}

impl FromStr for JourneyTime {
    type Err = ScheduleApplyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut time = Self {
            hour: 0,
            minute: 0,
            half: false,
        };
        let hour = &s[0..2];
        let min = &s[2..4];

        time.hour = hour
            .parse()
            .map_err(|_| ScheduleApplyError::InvalidJourneyTime(s.to_string()))?;
        time.minute = min
            .parse()
            .map_err(|_| ScheduleApplyError::InvalidJourneyTime(s.to_string()))?;

        if let Some(c) = s.chars().nth(4) {
            if c == 'H' {
                time.half = true;
            } else if c == ' ' {
                time.half = false;
            } else {
                return Err(ScheduleApplyError::InvalidJourneyTime(s.to_string()));
            }
        }
        Ok(time)
    }
}

#[allow(clippy::too_many_arguments)]
fn bs_record_to_schedule(
    schedule: &mut Schedule,
    uid: &str,
    date_runs_from: &str,
    date_runs_to: &str,
    days_run: &str,
    bank_holiday_running: &char,
    train_status: &char,
    train_category: &str,
    train_identity: &str,
    portion_id: &char,
    power_type: &str,
    timing_load: &str,
    speed: &str,
    operating_characteristics: &str,
    seating_class: &char,
    sleepers: &char,
    reservations: &char,
    catering_code: &str,
    stp_indicator: &char,
) -> Result<(), ScheduleApplyError> {
    schedule.train_uid = uid.to_string();
    schedule.runs_from = NaiveDate::parse_from_str(date_runs_from, "%y%m%d")
        .map_err(|_| ScheduleApplyError::InvalidScheduleDate(date_runs_from.to_string()))?;
    schedule.runs_to = NaiveDate::parse_from_str(date_runs_to, "%y%m%d")
        .map_err(|_| ScheduleApplyError::InvalidScheduleDate(date_runs_to.to_string()))?;
    schedule.days_run = DaysRun::from_bits(
        u8::from_str_radix(days_run, 2)
            .map_err(|_| ScheduleApplyError::InvalidDaysRun(days_run.to_string()))?,
    )
    .ok_or(ScheduleApplyError::InvalidDaysRun(days_run.to_string()))?;
    schedule.bank_holiday_running = match bank_holiday_running {
        'X' => BankHolidayRunning::NotOnSpecificBankHolidayMondays,
        'G' => BankHolidayRunning::NotOnGlasgowBankHolidays,
        _ => BankHolidayRunning::RunsNormally,
    };
    schedule.train_status = match train_status {
        ' ' => TrainStatus::NotSpecified,
        'B' => TrainStatus::Bus,
        'F' => TrainStatus::Freight,
        'P' => TrainStatus::PassengerAndParcels,
        'S' => TrainStatus::Ship,
        'T' => TrainStatus::Trip,
        '1' => TrainStatus::STPPassengerAndParcels,
        '2' => TrainStatus::STPFreight,
        '3' => TrainStatus::STPTrip,
        '4' => TrainStatus::STPShip,
        '5' => TrainStatus::STPBus,
        _ => return Err(ScheduleApplyError::InvalidTrainStatus(*train_status)),
    };
    schedule.train_category = match train_category {
        "  " => TrainCategory::NotSpecified,
        "OL" => TrainCategory::LondonUnderground,
        "OU" => TrainCategory::UnadvertisedOrdinaryPassenger,
        "OO" => TrainCategory::OrdinaryPassenger,
        "OS" => TrainCategory::StaffTrain,
        "OW" => TrainCategory::Mixed,
        "XC" => TrainCategory::ChannelTunnel,
        "XD" => TrainCategory::Sleeper,
        "XI" => TrainCategory::International,
        "XR" => TrainCategory::Motorail,
        "XU" => TrainCategory::UnadvertisedExpress,
        "XX" => TrainCategory::ExpressPassenger,
        "XZ" => TrainCategory::SleeperDomestic,
        "BR" => TrainCategory::BusReplacementDueToEngineering,
        "BS" => TrainCategory::BusWTTService,
        "SS" => TrainCategory::Ship,
        "EE" => TrainCategory::EmptyCoachingStock,
        "EL" => TrainCategory::ECSLondonUnderground,
        "ES" => TrainCategory::ECSAndStaff,
        "JJ" => TrainCategory::Postal,
        "PM" => TrainCategory::PostOfficeControlledParcels,
        "PP" => TrainCategory::Parcels,
        "PV" => TrainCategory::EmptyNPCCS,
        "DD" => TrainCategory::Departmental,
        "DH" => TrainCategory::CivilEngineer,
        "DI" => TrainCategory::MechanicalAndElectricalEngineer,
        "DQ" => TrainCategory::Stores,
        "DT" => TrainCategory::Test,
        "DY" => TrainCategory::SignalAndTelecommunicationsEngineer,
        "ZB" => TrainCategory::LocomotiveAndBrakeVan,
        "ZZ" => TrainCategory::LightLocomotive,
        "J2" => TrainCategory::RfDAutomotiveComponents,
        "H2" => TrainCategory::RfDAutomotiveVehicles,
        "J3" => TrainCategory::RfDEdibleProducts,
        "J4" => TrainCategory::RfDIndustrialMinerals,
        "J5" => TrainCategory::RfDChemicals,
        "J6" => TrainCategory::RfDBuildingMaterials,
        "J8" => TrainCategory::RfDGeneralMerchandise,
        "H8" => TrainCategory::RfDEuropean,
        "J9" => TrainCategory::RfDFreightlinerContracts,
        "H9" => TrainCategory::RfDFreightlinerOther,
        "A0" => TrainCategory::CoalDistributive,
        "E0" => TrainCategory::CoalElectricityMGR,
        "B0" => TrainCategory::CoalOtherAndNuclear,
        "B1" => TrainCategory::Metals,
        "B4" => TrainCategory::Aggregates,
        "B5" => TrainCategory::DomesticAndIndustrialWaste,
        "B6" => TrainCategory::BuildingMaterials,
        "B7" => TrainCategory::PetroleumProducts,
        "H0" => TrainCategory::RfDEuropeanChannelTunnelMixed,
        "H1" => TrainCategory::RfDEuropeanChannelTunnelIntermodal,
        "H3" => TrainCategory::RfDEuropeanChannelTunnelAutomotive,
        "H4" => TrainCategory::RfDEuropeanChannelTunnelContractServices,
        "H5" => TrainCategory::RfDEuropeanChannelTunnelHaulmark,
        "H6" => TrainCategory::RfDEuropeanChannelTunnelJointVenture,
        _ => {
            return Err(ScheduleApplyError::InvalidTrainCategory(
                train_category.to_string(),
            ))
        }
    };
    schedule.headcode = train_identity.trim().to_string();
    schedule.portion_id = *portion_id;
    schedule.power_type = match power_type.trim() {
        "" => PowerType::NotSpecified,
        "D" => PowerType::Diesel,
        "DEM" => PowerType::DieselElectricMultipleUnit,
        "DMU" => PowerType::DieselMechanicalMultipleUnit,
        "E" => PowerType::Electric,
        "ED" => PowerType::ElectroDiesel,
        "EML" => PowerType::EMUPlusLocomotive,
        "EMU" => PowerType::ElectricMultipleUnit,
        "HST" => PowerType::HighSpeedTrain,
        _ => return Err(ScheduleApplyError::InvalidPowerType(power_type.to_string())),
    };
    schedule.timing_load = if schedule.power_type == PowerType::DieselMechanicalMultipleUnit
        || schedule.power_type == PowerType::DieselElectricMultipleUnit
    {
        match timing_load.trim() {
            "" => TimingLoad::NotSpecified,
            "69" => TimingLoad::Class17201721Or1722,
            "A" => TimingLoad::Class141To144,
            "E" => TimingLoad::Class158168170Or175,
            "N" => TimingLoad::Class1650,
            "S" => TimingLoad::Class150153155Or156,
            "T" => TimingLoad::Class1651Or166,
            "V" => TimingLoad::Class220Or221,
            "X" => TimingLoad::Class159,
            "D1" => TimingLoad::DMUPowerCarTrailer,
            "D2" => TimingLoad::DMU2PowerCarsTrailer,
            "D3" => TimingLoad::DMUPowerTwin,
            _ => {
                if let Ok(n) = timing_load.trim().parse::<u16>() {
                    TimingLoad::SpecificClass(n)
                } else {
                    return Err(ScheduleApplyError::InvalidTimingLoad(
                        timing_load.to_string(),
                    ));
                }
            }
        }
    } else if schedule.power_type == PowerType::ElectricMultipleUnit {
        match timing_load.trim() {
            "" => TimingLoad::NotSpecified,
            "AT" => TimingLoad::AcceleratedTimings,
            "E" => TimingLoad::Class458,
            "0" => TimingLoad::Class380,
            "506" => TimingLoad::Class3501110MPH,
            _ => {
                if let Ok(n) = timing_load.trim().parse::<u16>() {
                    TimingLoad::SpecificClass(n)
                } else {
                    return Err(ScheduleApplyError::InvalidTimingLoad(
                        timing_load.to_string(),
                    ));
                }
            }
        }
    } else if schedule.power_type == PowerType::Diesel
        || schedule.power_type == PowerType::Electric
        || schedule.power_type == PowerType::ElectroDiesel
    {
        if timing_load.trim().is_empty() {
            TimingLoad::NotSpecified
        } else if schedule.power_type == PowerType::Electric && timing_load.trim() == "325" {
            TimingLoad::Class325ElectricParcelsUnit
        } else if let Ok(n) = timing_load.trim().parse::<u16>() {
            TimingLoad::LoadInTonnes(n)
        } else {
            return Err(ScheduleApplyError::InvalidTimingLoad(
                timing_load.to_string(),
            ));
        }
    } else {
        TimingLoad::NotSpecified
    };
    schedule.speed = speed.trim().parse().ok().unwrap_or(0);
    for c in operating_characteristics.chars() {
        match c {
            'B' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::VacuumBraked),
            'C' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::TimedAt100MPH),
            'D' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::DOOCoachingStockTrains),
            'E' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::ConveysMark4Coaches),
            'G' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::GuardRequired),
            'M' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::TimedAt110MPH),
            'P' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::PushPullTrain),
            'Q' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::RunsAsRequired),
            'R' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::AirConditionedWithPASystem),
            'S' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::SteamHeated),
            'Y' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::RunsToTerminalsAsRequired),
            'Z' => schedule
                .operating_characteristics
                .push(OperatingCharacteristic::MayConveyTrafficToSB1CGauge),
            ' ' => (),
            _ => return Err(ScheduleApplyError::InvalidOperatingCharacteristic(c)),
        };
    }
    schedule.seating_class = match seating_class {
        ' ' => SeatingClass::FirstAndStandard,
        'B' => SeatingClass::FirstAndStandard,
        'S' => SeatingClass::StandardOnly,
        _ => return Err(ScheduleApplyError::InvalidSeatingClass(*seating_class)),
    };
    schedule.sleepers = match sleepers {
        'B' => Sleepers::FirstAndStandard,
        'F' => Sleepers::FirstOnly,
        'S' => Sleepers::StandardOnly,
        ' ' => Sleepers::NotSpecified,
        _ => return Err(ScheduleApplyError::InvalidSleepers(*sleepers)),
    };
    schedule.reservations = match reservations {
        'A' => Reservations::Compulsory,
        'E' => Reservations::CompulsoryForBicycles,
        'R' => Reservations::Recommended,
        'S' => Reservations::Possible,
        ' ' => Reservations::NotSpecified,
        _ => return Err(ScheduleApplyError::InvalidReservations(*reservations)),
    };
    for c in catering_code.chars() {
        match c {
            'C' => schedule.catering.push(Catering::BuffetService),
            'F' => schedule.catering.push(Catering::RestaurantCarForFirstClass),
            'H' => schedule.catering.push(Catering::HotFood),
            'M' => schedule.catering.push(Catering::MealForFirstClass),
            'P' => schedule.catering.push(Catering::WheelchairReservations),
            'R' => schedule.catering.push(Catering::Restaurant),
            'T' => schedule.catering.push(Catering::TrolleyService),
            ' ' => (),
            _ => return Err(ScheduleApplyError::InvalidCateringCode(c)),
        };
    }
    schedule.stp_indicator = match stp_indicator {
        'C' => STPIndicator::STPCancellationOfPermanentAssociation,
        'N' => STPIndicator::NewSTPAssociation,
        'O' => STPIndicator::STPOverlayOfPermanentAssociation,
        'P' => STPIndicator::PermanentAssociation,
        _ => return Err(ScheduleApplyError::InvalidSTPIndicator(*stp_indicator)),
    };
    Ok(())
}
