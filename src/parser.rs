use std::io::{self, prelude::*};

use crate::types::*;
use thiserror::Error;

/// An error that occurred during parsing a CIF file.
#[derive(Debug, Error)]
pub enum CIFParseError {
    #[error("error at line {0}: {1}")]
    AtLine(usize, Box<CIFRecordParseError>),
    #[error("failed to read CIF file")]
    Read(#[from] io::Error),
}

/// Parse a CIF file into a programmatic [`CIFFile`].
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

        let record: CIFRecord = record_raw
            .parse()
            .map_err(|e| CIFParseError::AtLine(line, Box::new(e)))?;
        file.records_mut().push(record.clone());

        if let CIFRecord::Trailer = record {
            break;
        }
        line += 1;
    }
    Ok(file)
}
