//! Error
//! 
//! Functionality for error handling
//! Production code uses strict error handling
//! Test code can override Error and Result as follows
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     type Error = Box<dyn std::error::Error>;
//!     type Result<T> = core::result::Result<T,Error>;
//!     use super::*;
//! 
//!     #[test]
//!     fn test_x() -> Result<()> {
//!         // -- Setup and fixtures
//!         // -- Exec 
//!         // -- Check
//!     }
//! 
//! }
//! let x = some_function();
//! println!("{}", x);
//! ```
//! In contrast, production code has errors like this
//! ```ignore
//! #[derive(Debug, From)]
//! pub enum Error {
//!   IndexOutOfBounds { actual: usize, max: usize},
//! 
//!   #[from]
//!   SerdeJson(serde_json::Error)
//! }
//! ``` 
//! 


use core::fmt;

use derive_more::{From, Display};
use serde::Serialize;

use crate::individual_template::TemplateError;

// can be used for test modules pub type Error = Box<dyn std::err::Err>;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Serialize)]
pub enum Error {
    #[from]
    Custom(String),
    WhiteSpaceError{ msg: String},
    TranscriptError{ msg: String},
    LabelTooShort{ label: String, actual: usize, min: usize},
    EmptyLabel,
    ForbiddenLabelChar{ c: char, label: String},
    MalformedLabel{ label: String },
    MalformedDiseaseLabel{ label: String},
    EditError{ msg: String },
    TermIdError{ msg: String },
    HpIdNotFound{ id: String },
    ObsoleteTermId{ id: String, replacement: String },
    WrongLabel{ id:String, actual: String, expected: String},
    EmptyField{field_name: String},
    CurieError{ msg: String},
    PmidError{msg: String},
    DiseaseIdError{msg: String },
    HgncError{msg: String},
    HgvsError{msg: String},
    HeaderError{ msg: String},
    UnrecognizedValue{value: String, column_name: String },
    TemplateError{ msg: String },
    TermError{msg: String},
    AgeParseError{msg: String},
    DeceasedError{msg: String},
    SexFieldError{msg: String},
    SeparatorError{msg: String}
   
    // arrange according to module
    // -- pptcolumn


    /* -- Externals
    #[from]
    #[derive(serde::Serialize)]
    Io(std::io::Error), */
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }

    pub fn forbidden_character<T>(c: char, label: T) -> Self
        where T: Into<String> {
        Self::ForbiddenLabelChar { c, label: label.into() }
    }

    pub fn leading_ws<T>(value: T) -> Self
        where T: Into<String> {
        Self::WhiteSpaceError { msg: format!("Leading whitespace in '{}'", value.into()) }
    }

    pub fn trailing_ws<T>(value: T) -> Self
        where T: Into<String> {
            Self::WhiteSpaceError { msg:  format!("Trailing whitespace in '{}'", value.into()) }
    }

    pub fn column_not_found(colname: impl Into<String>) -> Self {
        Error::TemplateError{msg: format!("Could not find column {}", colname.into())}
    }

    pub fn row_index_error(idx: usize, rowcount: usize) -> Self {
        Error::TemplateError { msg: format!("Attempt to index row at index {idx} with row count {rowcount}") }
    }

    pub fn column_index_error(idx: usize, colcount: usize) -> Self {
        Error::TemplateError { msg: format!("Attempt to index column at index {idx} with column count {colcount}") }
    }

    pub fn short_label<T>(value: T, actual: usize, min: usize) -> Self 
        where T: Into<String> {
            Self::LabelTooShort { label:value.into(), actual, min }
    }

    pub fn lacks_transcript_version<T>(tx: T) -> Self
        where T: Into<String> {
            let msg =  format!("Transcript '{}' is missing a version", tx.into());
            Self::TranscriptError { msg: msg }
    }

    pub fn unrecognized_transcript_prefix<T>(tx: T) -> Self
        where T: Into<String> {
            let msg = format!("Unrecognized transcript prefix '{}'", tx.into());
            Self::TranscriptError { msg }
    }

    pub fn termid_parse_error<T>(identifier: T) -> Self 
        where T: Into<String>
    {
        Error::TermIdError{msg: format!("Failed to parse TermId: {}", identifier.into())}
    }

    pub fn sex_field_error<T>(val: T) -> Self  where T: Into<String> {
        Error::SexFieldError { msg: format!("Malformed sex field entry '{}'",val.into()) }
    }

    pub fn separator<T>(val: T) -> Self  where T: Into<String> {
        Error::SexFieldError { msg: format!("Malformed separator entry '{}'",val.into()) }
    }

    pub fn cannot_delete_header(row: usize) -> Self {
        let msg = format!("Cannot delete row {row} (header)");
        Error::EditError { msg }
    }

    pub fn delete_beyond_max_row(row: usize, max_row: usize) -> Self {
        let msg = format!("Attempt to delete row {row} in columns with {max_row} rows");
        Error::EditError { msg }
    }
    
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        match self {
            Error::LabelTooShort { label, actual, min } => {
                write!(fmt, "Label '{}' is too short ({} < required {})", label, actual, min)
            },
            Error::HpIdNotFound { id } => {
                write!(fmt, "Not able to find HPO TermId: {id}")
            },
            Error::ObsoleteTermId { id , replacement } => {
                write!(fmt, "Obsolete HPO TermId: {id}; replace with {replacement}.")
            },
            Error::MalformedLabel { label } => {
                write!(fmt, "Malformed label: '{label}'")
            },
            Error::MalformedDiseaseLabel { label } => {
                write!(fmt, "Malformed disease label: '{label}'")
            },
            Error::ForbiddenLabelChar { c, label } => {
                write!(fmt, "Forbidden character '{c}' found in label '{label}'")
            }
            Error::EmptyLabel  => {
                write!(fmt, "Empty label")
            },
            Error::EmptyField { field_name } => {
                write!(fmt, "{field_name} field is empty")
            },
            Error::TermIdError { msg }
            | Error::WhiteSpaceError { msg }
            | Error::HgvsError { msg }
            | Error::PmidError { msg }
            | Error::CurieError { msg }
            | Error::EditError { msg }
            | Error::DiseaseIdError { msg }
            | Error::TranscriptError { msg }
            | Error::DeceasedError { msg }
            | Error::TemplateError { msg }
            | Error::SexFieldError { msg }
            | Error::SeparatorError { msg }
            | Error::AgeParseError { msg } => 
                write!(fmt, "{msg}"),
            _ =>  write!(fmt, "{self:?}")
        }
    }
}

impl std::error::Error for Error {}