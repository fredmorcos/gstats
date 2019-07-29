use crate::id::Id;
use crate::txref::TxRef;
use std::fmt::{self, Display};

/// A Transaction
pub struct Transaction {
    /// Transaction ID
    pub id: Id,
    /// Left transaction reference
    pub left: TxRef,
    /// Right transaction reference
    pub right: TxRef,
    /// Transaction timestamp
    pub timestamp: usize,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Tx:{} <Left {}> <Right {}> ({})",
            self.id, self.left, self.right, self.timestamp
        )
    }
}
