use std::error::Error;
use std::fmt::Error as FmtError;
use std::clone::Clone;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use toml;

/**
 * Kind of store error
 */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoreErrorKind {
    FileError,
    IdLocked,
    IdNotFound,
    OutOfMemory,
    FileNotFound,
    FileNotCreated,
    StorePathExists,
    StorePathCreate,
    LockPoisoned,
    EntryAlreadyBorrowed
        // maybe more
}

fn store_error_type_as_str(e: &StoreErrorKind) -> &'static str {
    match e {
        &StoreErrorKind::FileError       => "File Error",
        &StoreErrorKind::IdLocked        => "ID locked",
        &StoreErrorKind::IdNotFound      => "ID not found",
        &StoreErrorKind::OutOfMemory     => "Out of Memory",
        &StoreErrorKind::FileNotFound    => "File corresponding to ID not found",
        &StoreErrorKind::FileNotCreated  => "File corresponding to ID could not be created",
        &StoreErrorKind::StorePathExists => "Store path exists",
        &StoreErrorKind::StorePathCreate => "Store path create",
        &StoreErrorKind::LockPoisoned
            => "The internal Store Lock has been poisoned",
            &StoreErrorKind::EntryAlreadyBorrowed => "Entry is already borrowed",
    }
}

impl Display for StoreErrorKind {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "{}", store_error_type_as_str(self)));
        Ok(())
    }

}

/**
 * Store error type
 */
#[derive(Debug)]
pub struct StoreError {
    err_type: StoreErrorKind,
    cause: Option<Box<Error>>,
}

impl StoreError {

    /**
     * Build a new StoreError from an StoreErrorKind, optionally with cause
     */
    pub fn new(errtype: StoreErrorKind, cause: Option<Box<Error>>)
        -> StoreError
        {
            StoreError {
                err_type: errtype,
                cause: cause,
            }
        }

    /**
     * Get the error type of this StoreError
     */
    pub fn err_type(&self) -> StoreErrorKind {
        self.err_type.clone()
    }

}

impl Display for StoreError {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        try!(write!(fmt, "[{}]", store_error_type_as_str(&self.err_type.clone())));
        Ok(())
    }

}

impl Error for StoreError {

    fn description(&self) -> &str {
        store_error_type_as_str(&self.err_type.clone())
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e)
    }

}


#[derive(Clone)]
pub enum ParserErrorKind {
    TOMLParserErrors,
    MissingMainSection,
    MissingVersionInfo,
}

pub struct ParserError {
    kind: ParserErrorKind,
    cause: Option<Box<Error>>,
}

impl ParserError {

    pub fn new(k: ParserErrorKind, cause: Option<Box<Error>>) -> ParserError {
        ParserError {
            kind: k,
            cause: cause,
        }
    }

}

impl Debug for ParserError {

    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{:?}", self.description()));
        Ok(())
    }

}

impl Display for ParserError {

    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{}", self.description()));
        Ok(())
    }

}

impl Error for ParserError {

    fn description(&self) -> &str {
        match self.kind {
            ParserErrorKind::TOMLParserErrors   => "Several TOML-Parser-Errors",
            ParserErrorKind::MissingMainSection => "Missing main section",
            ParserErrorKind::MissingVersionInfo => "Missing version information in main section",
        }
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e)
    }

}

