use crate::error::LSError;
use crate::id::Id;
use crate::Transaction;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

/// A reference to a transaction
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxRef {
    /// Reference to the root transaction
    Root,
    /// Reference to any other "normal" transaction
    Tx(Id),
}

impl Display for TxRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TxRef::Root => write!(f, "Root"),
            TxRef::Tx(id) => write!(f, "Tx:{}", id),
        }
    }
}

// Improve the verbosity of the debug output
impl Debug for TxRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<Id> for TxRef {
    /// Converts the special-case of a reference to ID=1 to refer to a special Root transaction,
    /// otherwise refers to a normal transaction
    fn from(id: Id) -> Self {
        if *id == 1 {
            TxRef::Root
        } else {
            TxRef::Tx(id)
        }
    }
}

impl TxRef {
    /// Check if a transaction reference is valid, a reference is invalid if it refers to a
    /// transaction index that is outside the bounds of the graph
    pub fn is_valid(
        self,
        id: Id,
        len: usize,
        err: impl Fn(Id, Id) -> LSError,
    ) -> Result<(), LSError> {
        match self {
            TxRef::Root => Ok(()),
            TxRef::Tx(ref_id) => {
                // The index the transaction reference is referencing
                let index: usize = ref_id.into();

                if index >= len {
                    // The index is outside the graph bounds, construct the error and return it
                    Err(err(id, ref_id))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Calculate the depth of a transaction in the graph: the depth is the shortest path from the
    /// transaction to the Root transaction
    pub fn depth(self, graph: &[Transaction], cache: &mut HashMap<Id, usize>) -> usize {
        match self {
            TxRef::Root => 0,
            TxRef::Tx(id) => {
                if let Some(depth) = cache.get(&id) {
                    // The depth of the current transaction is in the cache, return it
                    return *depth;
                }

                // Get the transaction
                let index: usize = id.into();
                let tx = &graph[index];

                // Get the depths of the left and right transactions
                let left_depth = tx.left.depth(graph, cache);
                let right_depth = tx.right.depth(graph, cache);

                // Insert the shorter depth into the cache as the depth of current transaction and
                // return it
                if left_depth < right_depth {
                    cache.insert(id, left_depth + 1);
                    left_depth + 1
                } else {
                    cache.insert(id, right_depth + 1);
                    right_depth + 1
                }
            }
        }
    }
}
