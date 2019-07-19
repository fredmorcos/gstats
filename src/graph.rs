#![warn(clippy::all)]

use crate::id::{Id, NonRootId};
use crate::transaction::{self, Transaction};
use derive_more::Display;
use std::convert::TryFrom;
use std::io::{self, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::str::FromStr;

/// Errors that can happen when dealing with graphs.
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "IO Error: {}", "_0")]
    IO(io::Error),

    #[display(fmt = "Missing number of transactions")]
    MissingNumberOfTransactions,

    #[display(fmt = "Invalid number of transactions: {}", "_0")]
    InvalidNumberOfTransactions(ParseIntError),

    #[display(fmt = "Too many transactions")]
    TooManyTransactions,

    #[display(fmt = "Too little transactions")]
    TooLittleTransactions,

    #[display(fmt = "Invalid transaction: {}", "_0")]
    InvalidTransaction(transaction::Error),

    #[display(fmt = "Invalid left ref to {} on Tx:{} max={}", "_1", "_0", "_2")]
    InvalidLeft(NonRootId, Id, usize),

    #[display(fmt = "Invalid right ref to {} on Tx:{} max={}", "_1", "_0", "_2")]
    InvalidRight(NonRootId, Id, usize),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<transaction::Error> for Error {
    fn from(e: transaction::Error) -> Self {
        Error::InvalidTransaction(e)
    }
}

/// Primary `Graph` data structure.
#[derive(PartialEq, Eq, Debug)]
pub struct Graph {
    /// The list of transactions.
    inner: Vec<Transaction>,
}

impl Graph {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    pub fn transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.inner.iter()
    }
}

impl<R: Read> TryFrom<BufReader<R>> for Graph {
    type Error = Error;

    fn try_from(input: BufReader<R>) -> Result<Self, Self::Error> {
        let mut iter = input.lines();

        // Read the expected number of transactions.
        let n_transactions = match iter.next() {
            Some(n) => n?,
            None => return Err(Error::MissingNumberOfTransactions),
        };

        let n_transactions = match usize::from_str(&n_transactions) {
            Ok(n) => n,
            Err(e) => return Err(Error::InvalidNumberOfTransactions(e)),
        };

        let mut graph = Graph::with_capacity(n_transactions);

        // Iterate over input lines.
        for (i, line) in iter.enumerate() {
            if i + 1 > n_transactions {
                // The number of transactions read so far exceeds the expected number.
                return Err(Error::TooManyTransactions);
            }

            // Current transaction's ID.
            let id = i + 2;

            // Parse the transaction.
            let t = Transaction::try_from((id, &line?))?;

            let max = n_transactions + 1;

            // Check the transaction's left reference.
            let left: usize = t.left().into();
            if left > max {
                return Err(Error::InvalidLeft(t.id(), t.left(), max));
            }

            // Check the transaction's right reference.
            let right: usize = t.right().into();
            if right > max {
                return Err(Error::InvalidRight(t.id(), t.right(), max));
            }

            graph.inner.push(t);
        }

        if graph.inner.len() < n_transactions {
            // The number of transactions read is lower than the expected number.
            return Err(Error::TooLittleTransactions);
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod graph_tests {
    use super::{Error, Graph};
    use crate::id::{Id, NonRootId};
    use crate::transaction::Transaction;
    use std::convert::TryFrom;
    use std::io::BufReader;

    fn graph() -> Graph {
        let second_id = NonRootId::try_from(2).unwrap();
        let second_left = Id::try_from(1).unwrap();
        let second_right = Id::try_from(1).unwrap();

        let third_id = NonRootId::try_from(3).unwrap();
        let third_left = Id::try_from(2).unwrap();
        let third_right = Id::try_from(1).unwrap();

        Graph {
            inner: vec![
                Transaction::new(second_id, second_left, second_right, 120),
                Transaction::new(third_id, third_left, third_right, 130),
            ],
        }
    }

    #[test]
    fn parse_success() {
        let input = String::from("2\n1 1 120\n2 1 130");
        let input = input.as_bytes();
        let res = Graph::try_from(BufReader::new(input)).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res, graph());
    }

    #[test]
    fn parse_fail() {
        let input = String::from("\n1 1 120\n2 1 130");
        let input = input.as_bytes();
        let res = Graph::try_from(BufReader::new(input));
        let parse_err = usize::from_str_radix("", 10).err().unwrap();

        match res {
            Err(Error::InvalidNumberOfTransactions(e)) => assert_eq!(e, parse_err),
            Err(e) => panic!("Unexpected error type: {}", e),
            Ok(_) => panic!("Unexpected success"),
        }
    }
}
