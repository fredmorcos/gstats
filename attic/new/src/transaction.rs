#![warn(clippy::all)]

//! Data structures for dealing with `Transaction`s.

use crate::id::{self, Id};
use derive_more::Display;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::num::ParseIntError;
use std::str::FromStr;

/// Errors that can happen when dealing with transactions.
#[derive(PartialEq, Eq, Display)]
pub enum Error {
    #[display(fmt = "Missing left reference")]
    MissingLeft,

    #[display(fmt = "Missing right reference")]
    MissingRight,

    #[display(fmt = "Missing timestamp")]
    MissingTimestamp,

    #[display(fmt = "Invalid left reference: {}", "_0")]
    InvalidLeft(ParseIntError),

    #[display(fmt = "Invalid right reference: {}", "_0")]
    InvalidRight(ParseIntError),

    #[display(fmt = "Invalid timestamp: {}", "_0")]
    InvalidTimestamp(ParseIntError),

    #[display(fmt = "Invalid left id: {}", "_0")]
    InvalidLeftId(id::Error),

    #[display(fmt = "Invalid right id: {}", "_0")]
    InvalidRightId(id::Error),
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// The `Transaction` structure with two references for left and right and a timestamp.
#[derive(PartialEq, Eq, Display)]
#[display(fmt = "Tx<{}, {}, {}>", left, right, timestamp)]
pub struct Transaction {
    left: Id,
    right: Id,
    timestamp: usize,
}

impl Transaction {
    pub fn new(left: Id, right: Id, timestamp: usize) -> Self {
        Self {
            left,
            right,
            timestamp,
        }
    }

    pub fn left(&self) -> Id {
        self.left
    }

    pub fn right(&self) -> Id {
        self.right
    }

    pub fn timestamp(&self) -> usize {
        self.timestamp
    }
}

impl FromStr for Transaction {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut iter = input.split_ascii_whitespace();

        // Read the left reference.
        let left = iter.next().ok_or(Error::MissingLeft)?;
        let left = usize::from_str(&left).map_err(Error::InvalidLeft)?;
        let left = Id::try_from(left).map_err(Error::InvalidLeftId)?;

        // Read the right reference.
        let right = iter.next().ok_or(Error::MissingRight)?;
        let right = usize::from_str(&right).map_err(Error::InvalidRight)?;
        let right = Id::try_from(right).map_err(Error::InvalidRightId)?;

        // Read the timestamp.
        let timestamp = iter.next().ok_or(Error::MissingTimestamp)?;
        let timestamp = usize::from_str(&timestamp).map_err(Error::InvalidTimestamp)?;

        Ok(Self::new(left, right, timestamp))
    }
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::Error;
    use super::Transaction;
    use crate::id::{self, Id};
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn parse_success() {
        let input = String::from("5 6 120");
        let res = Transaction::from_str(&input);
        let left = Id::try_from(5).unwrap();
        let right = Id::try_from(6).unwrap();
        assert_eq!(
            res,
            Ok(Transaction {
                left,
                right,
                timestamp: 120,
            })
        )
    }

    #[test]
    fn parse_success_root() {
        let input = String::from("1 1 120");
        let res = Transaction::from_str(&input);
        assert_eq!(
            res,
            Ok(Transaction {
                left: Id::Root,
                right: Id::Root,
                timestamp: 120,
            })
        )
    }

    #[test]
    fn parse_missing_left() {
        let input = String::from("");
        let res = Transaction::from_str(&input);
        assert_eq!(res, Err(Error::MissingLeft));
    }

    #[test]
    fn parse_missing_right() {
        let input = String::from("5");
        let res = Transaction::from_str(&input);
        assert_eq!(res, Err(Error::MissingRight));
    }

    #[test]
    fn parse_missing_timestamp() {
        let input = String::from("5 6");
        let res = Transaction::from_str(&input);
        assert_eq!(res, Err(Error::MissingTimestamp));
    }

    #[test]
    fn parse_invalid_left() {
        let input = String::from("abc");
        let res = Transaction::from_str(&input);
        assert_eq!(
            res,
            Err(Error::InvalidLeft(
                usize::from_str_radix("abc", 10).err().unwrap()
            ))
        );
    }

    #[test]
    fn parse_invalid_right() {
        let input = String::from("5 abc");
        let res = Transaction::from_str(&input);
        assert_eq!(
            res,
            Err(Error::InvalidRight(
                usize::from_str_radix("abc", 10).err().unwrap()
            ))
        );
    }

    #[test]
    fn parse_invalid_timestamp() {
        let input = String::from("5 6 abc");
        let res = Transaction::from_str(&input);
        assert_eq!(
            res,
            Err(Error::InvalidTimestamp(
                usize::from_str_radix("abc", 10).err().unwrap()
            ))
        );
    }

    #[test]
    fn parse_invalid_left_id() {
        let input = String::from("0 5 120");
        let res = Transaction::from_str(&input);
        assert_eq!(res, Err(Error::InvalidLeftId(id::Error::Invalid(0))));
    }

    #[test]
    fn parse_invalid_right_id() {
        let input = String::from("5 0 120");
        let res = Transaction::from_str(&input);
        assert_eq!(res, Err(Error::InvalidRightId(id::Error::Invalid(0))));
    }
}
