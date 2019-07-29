//! Graph structure with a few helpers

use crate::tx::Transaction;
use std::ops::Deref;

/// The Graph is just a list of transactions
pub struct Graph(Vec<Transaction>);

// Implement Deref to not have to re-implement every Vec method we need access to, and still
// disallow mutable access to the internals
impl Deref for Graph {
    type Target = Vec<Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Graph {
    pub fn with_capacity(cap: usize) -> Graph {
        Self(Vec::with_capacity(cap))
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.0.push(tx)
    }
}
