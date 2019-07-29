//! LedgerStats errors

use crate::id::Id;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::num::ParseIntError;

/// Error type for LedgerStats
pub enum LSError {
    /// Some IO error
    IO(io::Error),
    /// The given value for the number of transactions cannot be parsed
    NOfTxsMissing,
    /// Cannot parse the number of transactions
    NOfTxsInvalid(ParseIntError),
    /// More transactions than expected were found
    TooManyTxs(usize, usize),
    /// Less transactions than expected were found
    TooLittleTxs(usize, usize),
    /// A transaction is missing a left reference
    LRefMissing(Id),
    /// Cannot parse the left reference of a transaction
    LRefParse(Id, ParseIntError),
    /// A left transaction is pointing to a non-existent transaction
    LRefInvalid(Id, Id),
    /// A transaction is missing a right reference
    RRefMissing(Id),
    /// Cannot parse the right reference of a transaction
    RRefParse(Id, ParseIntError),
    /// A right transaction is pointing to a non-existent transaction
    RRefInvalid(Id, Id),
    /// A transaction is missing a timestamp
    TSMissing(Id),
    /// Cannot parse the timestamp of a transaction
    TSParse(Id, ParseIntError),
}

// Implement From so that we can use the `?` operator on io::Error in functions that return LSError
impl From<io::Error> for LSError {
    fn from(val: io::Error) -> Self {
        LSError::IO(val)
    }
}

impl Display for LSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LSError::*;

        match self {
            IO(err) => write!(f, "IO Error: {}", err),
            NOfTxsMissing => write!(f, "The number of transactions is missing"),
            NOfTxsInvalid(err) => write!(f, "Cannot parse the number of transactions: {}", err),
            TooManyTxs(given, found) => {
                write!(f, "Given {} transactions, but found >={}", given, found)
            }
            TooLittleTxs(given, found) => {
                write!(f, "Given {} transactions, but found {}", given, found)
            }
            LRefMissing(id) => write!(f, "Tx:{} is missing a left reference", id),
            LRefParse(id, err) => write!(f, "Cannot parse left reference for Tx:{}: {}", id, err),
            LRefInvalid(tx, reference) => {
                write!(f, "Tx:{} refers to invalid left {}", tx, reference)
            }
            RRefMissing(id) => write!(f, "Tx:{} is missing a right reference", id),
            RRefParse(id, err) => write!(f, "Cannot parse right reference for Tx:{}: {}", id, err),
            RRefInvalid(tx, reference) => {
                write!(f, "Tx:{} refers to invalid right {}", tx, reference)
            }
            TSMissing(id) => write!(f, "Tx:{} is missing a timestamp", id),
            TSParse(id, err) => write!(f, "Cannot parse timestamp for Tx:{}: {}", id, err),
        }
    }
}

// We want to print user-friendly looking errors
impl Debug for LSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Error for LSError {}
