//! Transaction ID and helpers to convert between IDs and indexes into the Graph's list of vertices

use std::fmt::{self, Debug, Display};
use std::ops::Deref;

/// The ID of a transaction (its line number in the database)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Id {
    fn from(index: usize) -> Self {
        // The ID (line number) is the index + 2, since IDs start at 1 rather than 0 (the Root
        // transaction) and the first line of the database input does not define a transaction
        Self(index + 2)
    }
}

impl Into<usize> for Id {
    fn into(self) -> usize {
        // The index of a transaction is the ID - 2
        self.0 - 2
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Id {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}
