#![warn(clippy::all)]

//! Transaction data structures.

use crate::id::{self, Id, NonRootId};
use derive_more::Display;
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::str::FromStr;

/// Errors that can happen when dealing with transactions.
#[derive(PartialEq, Eq, Debug, Display)]
pub enum Error {
    #[display(fmt = "Missing left reference")]
    MissingLeft,

    #[display(fmt = "Missing right reference")]
    MissingRight,

    #[display(fmt = "Missing timestamp")]
    MissingTimestamp,

    #[display(fmt = "Invalid Id: {}", "_0")]
    InvalidId(id::Error),

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

/// The `Transaction` structure with left and right references and a timestamp.
#[derive(PartialEq, Eq, Debug, Display)]
#[display(fmt = "Tx<{}, {}, {}, {}>", id, left, right, timestamp)]
pub struct Transaction {
    id: NonRootId,
    left: Id,
    right: Id,
    timestamp: usize,
}

impl Transaction {
    pub fn new(id: NonRootId, left: Id, right: Id, timestamp: usize) -> Self {
        Self {
            id,
            left,
            right,
            timestamp,
        }
    }

    pub fn id(&self) -> NonRootId {
        self.id
    }

    pub fn left(&self) -> Id {
        self.left
    }

    pub fn right(&self) -> Id {
        self.right
    }
}

impl TryFrom<(usize, &String)> for Transaction {
    type Error = Error;

    fn try_from((id, input): (usize, &String)) -> Result<Self, Self::Error> {
        let id = NonRootId::try_from(id).map_err(Error::InvalidId)?;

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

        Ok(Self::new(id, left, right, timestamp))
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::{Error, Transaction};
    use crate::id::{self, Id, NonRootId};
    use std::convert::TryFrom;

    #[test]
    fn parse_success() {
        let input = String::from("5 6 120");
        let res = Transaction::try_from((2, &input));
        let id = NonRootId::try_from(2).unwrap();
        let left = Id::try_from(5).unwrap();
        let right = Id::try_from(6).unwrap();
        assert_eq!(res, Ok(Transaction::new(id, left, right, 120)))
    }

    #[test]
    fn parse_success_root() {
        let input = String::from("1 1 120");
        let res = Transaction::try_from((2, &input));
        let id = NonRootId::try_from(2).unwrap();
        assert_eq!(res, Ok(Transaction::new(id, Id::Root, Id::Root, 120)))
    }

    #[test]
    fn parse_missing_left() {
        let input = String::from("");
        let res = Transaction::try_from((2, &input));
        assert_eq!(res, Err(Error::MissingLeft));
    }

    #[test]
    fn parse_missing_right() {
        let input = String::from("5");
        let res = Transaction::try_from((2, &input));
        assert_eq!(res, Err(Error::MissingRight));
    }

    #[test]
    fn parse_missing_timestamp() {
        let input = String::from("5 6");
        let res = Transaction::try_from((2, &input));
        assert_eq!(res, Err(Error::MissingTimestamp));
    }

    #[test]
    fn parse_invalid_left() {
        let input = String::from("abc");
        let res = Transaction::try_from((2, &input));
        let err = usize::from_str_radix("abc", 10).err().unwrap();
        assert_eq!(res, Err(Error::InvalidLeft(err)));
    }

    #[test]
    fn parse_invalid_right() {
        let input = String::from("5 abc");
        let res = Transaction::try_from((2, &input));
        let err = usize::from_str_radix("abc", 10).err().unwrap();
        assert_eq!(res, Err(Error::InvalidRight(err)));
    }

    #[test]
    fn parse_invalid_timestamp() {
        let input = String::from("5 6 abc");
        let res = Transaction::try_from((2, &input));
        let err = usize::from_str_radix("abc", 10).err().unwrap();
        assert_eq!(res, Err(Error::InvalidTimestamp(err)));
    }

    #[test]
    fn parse_invalid_left_id() {
        let input = String::from("0 5 120");
        let res = Transaction::try_from((2, &input));
        assert_eq!(res, Err(Error::InvalidLeftId(id::Error::Invalid)));
    }

    #[test]
    fn parse_invalid_right_id() {
        let input = String::from("5 0 120");
        let res = Transaction::try_from((2, &input));
        assert_eq!(res, Err(Error::InvalidRightId(id::Error::Invalid)));
    }
}
