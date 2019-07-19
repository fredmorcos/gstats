#![warn(clippy::all)]

use crate::id::{Id, NonRootId};
use crate::transaction::{self, Transaction};
use derive_more::Display;
use std::collections::{HashMap as Map, HashSet as Set};
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
#[derive(PartialEq, Eq, Debug, Default)]
pub struct Graph {
    /// The list of transactions.
    inner: Vec<Transaction>,

    /// The reverse references. A map from transaction `Id`s (including the Root
    /// transaction) to the set of transactions pointing to it, including the number of
    /// edges pointing to it.
    reverse: Map<Id, Set<NonRootId>>,
}

impl Graph {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            reverse: Map::with_capacity(cap + 1),
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

    fn references(&self, id: Id) -> Option<&Set<NonRootId>> {
        self.reverse.get(&id)
    }

    fn push(&mut self, transaction: Transaction) {
        // Insert a new entry for incoming references to the left reference of the
        // transaction.
        let left_references = self
            .reverse
            .entry(transaction.left())
            .or_insert_with(Set::new);

        // Insert the new source transaction for the incoming reference and increment the
        // number of incoming references.
        left_references.insert(transaction.id());

        // Insert a new entry for incoming references to the right reference of the
        // transaction.
        let right_references = self
            .reverse
            .entry(transaction.right())
            .or_insert_with(Set::new);

        // Insert the new source transaction for the incoming reference and increment the
        // number of incoming references.
        right_references.insert(transaction.id());

        self.inner.push(transaction);
    }

    /// Check whether the `Graph` is connected and acyclic.
    pub fn is_connected_acyclic(&self) -> Option<bool> {
        fn helper(graph: &Graph, vertex: Id, mut history: Set<Id>, visited: &mut Set<Id>) -> bool {
            if history.contains(&vertex) {
                return false;
            }

            history.insert(vertex);
            visited.insert(vertex);

            if let Some(references) = graph.references(vertex) {
                for next in references {
                    if !helper(graph, Id::Transaction(*next), history.clone(), visited) {
                        return false;
                    }
                }
            }

            true
        }

        let history = Set::new();
        let mut visited = Set::new();
        let res = helper(self, Id::Root, history, &mut visited);

        if visited.len() == self.len() + 1 {
            Some(res)
        } else {
            None
        }
    }

    /// Check whether the `Graph` is bipartite. Uses a two-coloring
    /// implementation. Assumes all vertices are reachable from the Root transaction.
    pub fn is_bipartite(&self) -> bool {
        fn helper(graph: &Graph, vertex: Id, color: bool, colors: &mut Map<Id, bool>) -> bool {
            if let Some(c) = colors.get(&vertex) {
                // If the current transaction is already colored and it does not match
                // with the prospective color, then the graph cannot be bipartite.
                if *c != color {
                    return false;
                }
            } else {
                // If the current transaction is not colored, insert its color into the
                // "visited"/"colored" transactions accumulator.
                colors.insert(vertex, color);
            }

            if let Some(references) = graph.references(vertex) {
                // Recursively follow in-references with the opposite color.
                for next in references {
                    if !helper(graph, Id::from(*next), !color, colors) {
                        return false;
                    }
                }
            }

            true
        }

        // Call the helper with the color accumulator starting at the Root. Due to the
        // structure of our graphs and the assumptions we make, every transaction should
        // be inverse-reachable from the Root.
        let mut colors = Map::new();
        helper(self, Id::Root, false, &mut colors)
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

            graph.push(t);
        }

        if graph.len() < n_transactions {
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
        let mut graph = Graph::default();

        graph.push(Transaction::new(
            NonRootId::try_from(2).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(1).unwrap(),
            120,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(3).unwrap(),
            Id::try_from(2).unwrap(),
            Id::try_from(1).unwrap(),
            130,
        ));

        graph
    }

    fn bp_graph() -> Graph {
        let mut graph = Graph::default();

        graph.push(Transaction::new(
            NonRootId::try_from(2).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(1).unwrap(),
            120,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(3).unwrap(),
            Id::try_from(2).unwrap(),
            Id::try_from(2).unwrap(),
            130,
        ));

        graph
    }

    fn cyclic_graph() -> Graph {
        let mut graph = Graph::default();

        graph.push(Transaction::new(
            NonRootId::try_from(2).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(3).unwrap(),
            120,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(3).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(4).unwrap(),
            130,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(4).unwrap(),
            Id::try_from(1).unwrap(),
            Id::try_from(2).unwrap(),
            130,
        ));

        graph
    }

    fn unconnected_graph() -> Graph {
        let mut graph = Graph::default();

        graph.push(Transaction::new(
            NonRootId::try_from(2).unwrap(),
            Id::try_from(3).unwrap(),
            Id::try_from(3).unwrap(),
            120,
        ));

        graph.push(Transaction::new(
            NonRootId::try_from(3).unwrap(),
            Id::try_from(2).unwrap(),
            Id::try_from(2).unwrap(),
            130,
        ));

        graph
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

    #[test]
    fn bipartite() {
        assert!(!graph().is_bipartite());
        assert!(bp_graph().is_bipartite());
    }

    #[test]
    fn connected_acyclic() {
        assert_eq!(graph().is_connected_acyclic(), Some(true));
        assert_eq!(bp_graph().is_connected_acyclic(), Some(true));
        assert_eq!(cyclic_graph().is_connected_acyclic(), Some(false));
        assert_eq!(unconnected_graph().is_connected_acyclic(), None);
    }
}
