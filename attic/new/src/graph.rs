#![warn(clippy::all)]

//! The main `Graph` data structures.

use crate::id::{self, Id, NonRootId};
use crate::transaction::{self, Transaction};
use derive_more::Display;
use either::Either;
use std::collections::{HashMap as Map, HashSet as Set};
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::io::Read;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::ops::Index;
use std::str::FromStr;

/// Errors that can happen when dealing with graphs.
#[derive(PartialEq, Eq, Display)]
pub enum Error {
    #[display(fmt = "Missing number of transactions")]
    MissingNumberOfTransactions,

    #[display(fmt = "Invalid number of transactions: {}", "_0")]
    InvalidNumberOfTransactions(ParseIntError),

    #[display(fmt = "Too many transactions")]
    TooManyTransactions,

    #[display(fmt = "Too little transactions")]
    TooLittleTransactions,

    #[display(fmt = "Invalid Id {}: {}", "_0", "_1")]
    InvalidId(usize, id::Error),

    #[display(fmt = "Invalid transaction at {}: {}", "_0", "_1")]
    InvalidTransaction(NonRootId, transaction::Error),

    #[display(fmt = "Invalid left ref to {} on Tx:{} max={}", "_1", "_0", "_2")]
    InvalidLeft(NonRootId, Id, usize),

    #[display(fmt = "Invalid right ref to {} on Tx:{} max={}", "_1", "_0", "_2")]
    InvalidRight(NonRootId, Id, usize),

    #[display(fmt = "Graph is cyclic")]
    Cyclic,
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct InRefs {
    inner: Set<NonRootId>,
    count: usize,
}

impl InRefs {
    pub fn count(&self) -> usize {
        self.count
    }

    /// Insert the new source transaction for the incoming reference and increment the
    /// number of incoming references.
    fn add(&mut self, id: NonRootId) {
        self.inner.insert(id);
        self.count += 1;
    }

    fn sources(&self) -> impl Iterator<Item = NonRootId> + '_ {
        self.inner.iter().copied()
    }
}

/// Primary `Graph` data structure.
#[derive(PartialEq, Eq, Debug)]
pub struct Graph {
    /// The list of transactions.
    inner: Vec<Transaction>,

    /// The "reverse"/"in" references. A map from transaction `Id`s (including the Root
    /// transaction) to the set of transactions pointing to it, as well asthe number of
    /// edges pointing to it.
    inrefs: Map<Id, InRefs>,
}

impl Graph {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            inrefs: Map::with_capacity(cap + 1),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn transactions(&self) -> impl Iterator<Item = Option<(NonRootId, &Transaction)>> {
        self.inner
            .iter()
            .enumerate()
            .map(|(i, t)| NonRootId::try_from(i + 2).ok().map(|id| (id, t)))
    }

    pub fn in_refs(&self, id: Id) -> Option<&InRefs> {
        self.inrefs.get(&id)
    }

    pub fn push(&mut self, id: NonRootId, transaction: Transaction) {
        // Insert a new entry for incoming references to the left reference of the
        // transaction.
        let left_inrefs = self
            .inrefs
            .entry(transaction.left())
            .or_insert_with(InRefs::default);

        // Insert the new source transaction for the incoming reference and increment the
        // number of incoming references.
        left_inrefs.add(id);

        // Insert a new entry for incoming references to the right reference of the
        // transaction.
        let right_inrefs = self
            .inrefs
            .entry(transaction.right())
            .or_insert_with(InRefs::default);

        // Insert the new source transaction for the incoming reference and increment the
        // number of incoming references.
        right_inrefs.add(id);

        self.inner.push(transaction);
    }

    /// Check whether the `Graph` is acyclic. Assumes all vertices are reachable from the
    /// Root transaction.
    pub fn is_acyclic(&self) -> bool {
        fn helper(graph: &Graph, vertex: Id, mut history: Set<Id>) -> bool {
            if history.contains(&vertex) {
                return false;
            }

            history.insert(vertex);

            if let Some(in_refs) = graph.in_refs(vertex) {
                for next in in_refs.sources() {
                    if !helper(graph, Id::Transaction(next), history.clone()) {
                        return false;
                    }
                }
            }

            true
        }

        let history = Set::new();
        helper(self, Id::Root, history)
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

            if let Some(inrefs) = graph.inrefs.get(&vertex) {
                // Recursively follow in-references with the opposite color.
                for next in inrefs.sources() {
                    if !helper(graph, Id::from(next), !color, colors) {
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

    /// Typical recursive depth implementation with a memoization/cache. Assumes the graph
    /// is acyclic.
    pub fn depth(&self, id: NonRootId, cache: &mut Map<NonRootId, usize>) -> usize {
        if let Some(depth) = cache.get(&id) {
            // The depth has already been computed due to some other depth lookup, just
            // return it from the cache.
            return *depth;
        }

        let transaction = &self[id];

        // Lookup the left reference and compute its depth. If the left reference refers
        // to the Root transaction then the current transaction's depth is 1, otherwise
        // it's the left transaction's depth + 1.
        let left_depth = if let Id::Transaction(left_id) = transaction.left() {
            self.depth(left_id, cache) + 1
        } else {
            1
        };

        // Lookup the right reference and compute its depth. If the right reference refers
        // to the Root transaction then the current transaction's depth is 1, otherwise
        // it's the right transaction's depth + 1.
        let right_depth = if let Id::Transaction(right_id) = transaction.right() {
            self.depth(right_id, cache) + 1
        } else {
            1
        };

        // The depth is the shorter path of the two possibilities. Store it in the cache
        // for future lookup.
        let depth = left_depth.min(right_depth);
        cache.insert(id, depth);
        depth
    }
}

impl<R: Read> TryFrom<BufReader<R>> for Graph {
    type Error = Either<io::Error, Error>;

    fn try_from(input: BufReader<R>) -> Result<Self, Self::Error> {
        let mut iter = input.lines();

        // Read the expected number of transactions.
        let n_transactions = match iter.next() {
            Some(Ok(n)) => n,
            Some(Err(e)) => return Err(Either::Left(e)),
            None => return Err(Either::Right(Error::MissingNumberOfTransactions)),
        };

        let n_transactions = match usize::from_str(&n_transactions) {
            Ok(n) => n,
            Err(e) => return Err(Either::Right(Error::InvalidNumberOfTransactions(e))),
        };

        let mut graph = Graph::with_capacity(n_transactions);

        // Iterate over input lines.
        for (i, line) in iter.enumerate() {
            if i + 1 > n_transactions {
                // The number of transactions read so far exceeds the expected number of
                // transactions.
                return Err(Either::Right(Error::TooManyTransactions));
            }

            // Create and validate the current transaction's ID.
            let id = i + 2;
            let id = NonRootId::try_from(id).map_err(|e| Either::Right(Error::InvalidId(id, e)))?;

            // Parse the transaction.
            let line = line.map_err(Either::Left)?;
            let t = Transaction::from_str(&line)
                .map_err(|e| Either::Right(Error::InvalidTransaction(id, e)))?;

            let max = n_transactions + 1;

            // Check the transaction's left reference.
            let left: usize = t.left().into();
            if left > max {
                return Err(Either::Right(Error::InvalidLeft(id, t.left(), max)));
            }

            // Check the transaction's right reference.
            let right: usize = t.right().into();
            if right > max {
                return Err(Either::Right(Error::InvalidRight(id, t.right(), max)));
            }

            graph.push(id, t);
        }

        if graph.len() < n_transactions {
            // The number of transactions read is lower than the expected number of
            // transactions.
            return Err(Either::Right(Error::TooLittleTransactions));
        }

        if !graph.is_acyclic() {
            return Err(Either::Right(Error::Cyclic));
        }

        Ok(graph)
    }
}

impl Index<NonRootId> for Graph {
    type Output = Transaction;

    fn index(&self, index: NonRootId) -> &Self::Output {
        let index: usize = index.into();
        &self.inner[index - 2]
    }
}

#[cfg(test)]
mod graph_tests {
    use super::Error;
    use super::{Graph, InRefs};
    use crate::id::{Id, NonRootId};
    use crate::transaction::Transaction;
    use either::Either;
    use std::collections::HashMap as Map;
    use std::convert::TryFrom;
    use std::io::BufReader;

    fn nonbp_graph() -> Graph {
        let second_left = Id::try_from(1).unwrap();
        let second_right = Id::try_from(1).unwrap();
        let third_left = Id::try_from(2).unwrap();
        let third_right = Id::try_from(1).unwrap();

        let mut inrefs_root = InRefs::default();
        inrefs_root.add(NonRootId::try_from(2).unwrap());
        inrefs_root.add(NonRootId::try_from(2).unwrap());
        inrefs_root.add(NonRootId::try_from(3).unwrap());

        let mut inrefs_second = InRefs::default();
        inrefs_second.add(NonRootId::try_from(3).unwrap());

        let mut inrefs = Map::new();
        inrefs.insert(Id::try_from(1).unwrap(), inrefs_root);
        inrefs.insert(Id::try_from(2).unwrap(), inrefs_second);

        Graph {
            inner: vec![
                Transaction::new(second_left, second_right, 120),
                Transaction::new(third_left, third_right, 130),
            ],
            inrefs,
        }
    }

    fn bp_graph() -> Graph {
        let second_left = Id::try_from(1).unwrap();
        let second_right = Id::try_from(1).unwrap();
        let third_left = Id::try_from(2).unwrap();
        let third_right = Id::try_from(2).unwrap();

        let mut inrefs_root = InRefs::default();
        inrefs_root.add(NonRootId::try_from(2).unwrap());
        inrefs_root.add(NonRootId::try_from(2).unwrap());

        let mut inrefs_second = InRefs::default();
        inrefs_second.add(NonRootId::try_from(3).unwrap());
        inrefs_second.add(NonRootId::try_from(3).unwrap());

        let mut inrefs = Map::new();
        inrefs.insert(Id::try_from(1).unwrap(), inrefs_root);
        inrefs.insert(Id::try_from(2).unwrap(), inrefs_second);

        Graph {
            inner: vec![
                Transaction::new(second_left, second_right, 120),
                Transaction::new(third_left, third_right, 130),
            ],
            inrefs,
        }
    }

    #[test]
    fn parse_success() {
        let input = String::from("2\n1 1 120\n2 1 130");
        let input = input.as_bytes();
        let res = Graph::try_from(BufReader::new(input));
        let model = nonbp_graph();

        match res {
            Err(Either::Left(e)) => panic!("IO Error: {}", e),
            Err(Either::Right(e)) => panic!("Unexpected Error: {}", e),
            Ok(res) => {
                assert_eq!(res, model);
                assert_eq!(res.len(), 2);
            }
        }
    }

    #[test]
    fn parse_fail() {
        let input = String::from("\n1 1 120\n2 1 130");
        let input = input.as_bytes();
        let res = Graph::try_from(BufReader::new(input));

        let parse_err = usize::from_str_radix("", 10).err().unwrap();

        match res {
            Err(Either::Left(e)) => panic!("IO Error: {}", e),
            Err(Either::Right(e)) => assert_eq!(e, Error::InvalidNumberOfTransactions(parse_err)),
            Ok(res) => panic!("Unexpected success: {:#?}", res),
        }
    }

    #[test]
    fn bipartite() {
        assert!(bp_graph().is_bipartite());
    }

    #[test]
    fn not_bipartite() {
        assert!(!nonbp_graph().is_bipartite());
    }
}
