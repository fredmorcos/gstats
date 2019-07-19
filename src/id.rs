#![warn(clippy::all)]

//! Data structures for IDs and references to transactions.

use derive_more::Display;
use std::convert::TryFrom;
use std::num::NonZeroUsize;

/// Errors when dealing with transaction IDs.
#[derive(PartialEq, Eq, Debug, Display)]
pub enum Error {
    #[display(fmt = "Invalid ID 0")]
    Invalid,

    #[display(fmt = "ID 1 is reserved for Root")]
    Reserved,
}

/// The ID of a transaction that is not the root. This mainly exists to index into the
/// `Graph` structure without the risk of trying to index the Root transaction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display(fmt = "Id({})", "_0")]
pub struct NonRootId(NonZeroUsize);

impl TryFrom<usize> for NonRootId {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Err(Error::Reserved),
            _ => Ok(Self(NonZeroUsize::new(value).ok_or(Error::Invalid)?)),
        }
    }
}

impl Into<usize> for NonRootId {
    fn into(self) -> usize {
        self.0.get()
    }
}

#[cfg(test)]
mod nonroot_id_tests {
    use super::{Error, NonRootId};
    use std::convert::TryFrom;

    #[test]
    fn valid_id() {
        let id = NonRootId::try_from(2).unwrap();
        let id: usize = id.into();
        assert_eq!(id, 2);
    }

    #[test]
    fn invalid_id() {
        assert_eq!(NonRootId::try_from(0), Err(Error::Invalid));
        assert_eq!(NonRootId::try_from(1), Err(Error::Reserved));
    }
}

/// The ID of a transaction and its index in the `Graph` vector of transactions. Since the
/// Root transaction has no representation beyond being the variant of an enum, then we
/// treat the special `Id` of 1 to be a reference to it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum Id {
    #[display(fmt = "Root")]
    Root,

    #[display(fmt = "Tx:{}", "_0")]
    Transaction(NonRootId),
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

#[cfg(test)]
mod id_tests {
    use super::{Id, NonRootId};
    use std::convert::TryFrom;

    #[test]
    fn transaction_id_from_nrid() {
        assert_eq!(
            Id::try_from(2),
            Ok(Id::Transaction(NonRootId::try_from(2).unwrap()))
        );
    }

    #[test]
    fn root_id() {
        assert_eq!(Id::try_from(1), Ok(Id::Root));
    }
}
