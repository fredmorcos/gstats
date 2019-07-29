#![warn(clippy::all)]

//! Data structures for `Id`s and references to transactions.

use derive_more::Display;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::num::NonZeroUsize;

/// Errors that can happen when dealing with transaction IDs.
#[derive(PartialEq, Eq, Display)]
pub enum Error {
    #[display(fmt = "Invalid Id {}", "_0")]
    Invalid(usize),
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// The ID of a transaction that is not the root. This mainly exists to index into the
/// `Graph` structure without the risk of trying to index the Root transaction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "Id({})", "_0")]
pub struct NonRootId(NonZeroUsize);

impl TryFrom<usize> for NonRootId {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 | 1 => Err(Error::Invalid(value)),
            _ => NonZeroUsize::new(value)
                .ok_or_else(|| Error::Invalid(value))
                .map(NonRootId),
        }
    }
}

impl Into<usize> for NonRootId {
    fn into(self) -> usize {
        self.0.get()
    }
}

impl Debug for NonRootId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod non_root_id_tests {
    use super::NonRootId;
    use std::convert::TryFrom;

    #[test]
    fn valid_id() {
        let id = NonRootId::try_from(2).unwrap();
        let id: usize = id.into();
        assert_eq!(id, 2);
    }

    #[test]
    #[should_panic]
    fn invalid_id() {
        let _ = NonRootId::try_from(1).unwrap();
    }
}

/// An `Id` refers to the ID of a transaction, but also to its index in the `Graph`'s
/// vector of transactions. Since the Root transaction has no representation beyond being
/// the variant of an enum, then we treat the special `Id` of 1 to be a reference to it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Id {
    #[display(fmt = "Root")]
    Root,

    #[display(fmt = "Tx:{}", "_0")]
    Transaction(NonRootId),
}

impl From<NonRootId> for Id {
    fn from(value: NonRootId) -> Self {
        Id::Transaction(value)
    }
}

impl TryFrom<usize> for Id {
    type Error = <NonRootId as TryFrom<usize>>::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Id::Root),
            _ => Ok(Id::Transaction(NonRootId::try_from(value)?)),
        }
    }
}

impl Into<usize> for Id {
    fn into(self) -> usize {
        match self {
            Id::Root => 1,
            Id::Transaction(id) => id.into(),
        }
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod id_tests {
    use super::Id;
    use super::NonRootId;
    use std::convert::TryFrom;

    #[test]
    fn transaction_id_from_nrid() {
        let nr_id = NonRootId::try_from(2).unwrap();
        let id = Id::try_from(nr_id);
        assert_eq!(id, Ok(Id::Transaction(nr_id)));
    }

    #[test]
    fn root_id() {
        let id = Id::try_from(1);
        assert_eq!(id, Ok(Id::Root));
    }
}
