use std::io::{self, prelude::*};

use crate::types::*;
use chrono::{NaiveDate, NaiveTime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CIFParseError {
    #[error("error at line {0}: {1}")]
    AtLine(usize, Box<CIFParseError>),
    #[error("failed to read CIF file")]
    Read(#[from] io::Error),
    #[error("invalid record type '{0}'")]
    InvalidRecordType(String),
    #[error("a record is garbled and cannot be parsed")]
    GarbledRecord,

    // Header specifics
    #[error("failed to parse date of extract")]
    FailedToParseDateOfExtract,
    #[error("failed to parse time of extract")]
    FailedToParseTimeOfExtract,
    #[error("the update indicator character is invalid")]
    InvalidUpdateIndicator,

    // TIPLOC specific
    #[error("invalid capitals identification for TIPLOC")]
    InvalidCapitalsIdentification,
    #[error("invalid national location code for TIPLOC")]
    InvalidNLC,
    #[error("invalid stanox code for TIPLOC")]
    InvalidStanox,
}

pub fn parse_cif<R: Read>(mut content_reader: R) -> Result<crate::types::CIFFile, CIFParseError> {
    let mut file = CIFFile::new();
    let mut buf = [0u8; 81];
    // human readable line of the input file
    let mut line = 1;
    loop {
        // read 80 character row + new line
        // the file should only contain ASCII characters, so we don't need to worry
        // about multi-byte characters
        content_reader.read_exact(&mut buf)?;

        // process buffer
        let record_raw = String::from_utf8_lossy(&buf[0..80]);
        let record_type = &record_raw[0..2];

        let record = match record_type {
            "HD" => parse_header(&record_raw),
            "TI" => parse_tiploc_insert(&record_raw),
            "TA" => parse_tiploc_amend(&record_raw),
            "TD" => parse_tiploc_delete(&record_raw),
            "AA" => Ok(CIFRecord::Trailer),
            "BS" => Ok(CIFRecord::Trailer),
            "BX" => Ok(CIFRecord::Trailer),
            "LO" => Ok(CIFRecord::Trailer),
            "LI" => Ok(CIFRecord::Trailer),
            "CR" => Ok(CIFRecord::Trailer),
            "LT" => Ok(CIFRecord::Trailer),
            "ZZ" => Ok(CIFRecord::Trailer),
            _ => return Err(CIFParseError::InvalidRecordType(record_type.to_string())),
        }
        .map_err(|e| CIFParseError::AtLine(line, Box::new(e)))?;
        file.records_mut().push(record);

        if record_type == "ZZ" {
            break;
        }
        line += 1;
    }
    Ok(file)
}

fn parse_header(record: &str) -> Result<CIFRecord, CIFParseError> {
    Ok(CIFRecord::Header {
        file_mainframe_identity: record[2..22].to_string(),
        date_of_extract: NaiveDate::parse_from_str(&record[22..28], "%y%m%d")
            .map_err(|_| CIFParseError::FailedToParseDateOfExtract)?,
        time_of_extract: NaiveTime::parse_from_str(&record[28..32], "%H%M")
            .map_err(|_| CIFParseError::FailedToParseDateOfExtract)?,
        current_file_reference: record[32..39].to_string(),
        last_file_reference: record[39..46].to_string(),
        update_indicator: match record.chars().nth(46).ok_or(CIFParseError::GarbledRecord)? {
            'U' => CIFUpdateIndicator::Update,
            'F' => CIFUpdateIndicator::Full,
            _ => return Err(CIFParseError::InvalidUpdateIndicator),
        },
        version: record.chars().nth(47).ok_or(CIFParseError::GarbledRecord)?,
        user_start_date: NaiveDate::parse_from_str(&record[48..54], "%y%m%d")
            .map_err(|_| CIFParseError::FailedToParseDateOfExtract)?,
        user_end_date: NaiveDate::parse_from_str(&record[54..60], "%y%m%d")
            .map_err(|_| CIFParseError::FailedToParseDateOfExtract)?,
    })
}

fn parse_tiploc_insert(record: &str) -> Result<CIFRecord, CIFParseError> {
    Ok(CIFRecord::TIPLOCInsert {
        tiploc: record[2..9].to_string(),
        capitals_identification: record[9..11]
            .parse::<u8>()
            .map_err(|_| CIFParseError::InvalidCapitalsIdentification)?,
        nlc: record[11..17]
            .parse::<u32>()
            .map_err(|_| CIFParseError::InvalidNLC)?,
        nlc_check_char: record.chars().nth(17).ok_or(CIFParseError::GarbledRecord)?,
        tps_description: record[18..44].to_string(),
        stanox: record[44..49]
            .parse::<u32>()
            .map_err(|_| CIFParseError::InvalidStanox)?,
        three_alpha_code: record[53..56].to_string(),
        nlc_description: record[56..72].to_string(),
    })
}

fn parse_tiploc_amend(record: &str) -> Result<CIFRecord, CIFParseError> {
    Ok(CIFRecord::TIPLOCAmend {
        tiploc: record[2..9].to_string(),
        capitals_identification: record[9..11]
            .parse::<u8>()
            .map_err(|_| CIFParseError::InvalidCapitalsIdentification)?,
        nlc: record[11..17]
            .parse::<u32>()
            .map_err(|_| CIFParseError::InvalidNLC)?,
        nlc_check_char: record.chars().nth(17).ok_or(CIFParseError::GarbledRecord)?,
        tps_description: record[18..44].to_string(),
        stanox: record[44..49]
            .parse::<u32>()
            .map_err(|_| CIFParseError::InvalidStanox)?,
        three_alpha_code: record[53..56].to_string(),
        nlc_description: record[56..72].to_string(),
        new_tiploc: record[72..79].to_string(),
    })
}

fn parse_tiploc_delete(record: &str) -> Result<CIFRecord, CIFParseError> {
    Ok(CIFRecord::TIPLOCDelete {
        tiploc: record[2..9].to_string(),
    })
}
