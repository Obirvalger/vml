use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecifiedBy {
    All,
    Tag,
    Parent(String),
    Name,
}

impl PartialOrd for SpecifiedBy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            SpecifiedBy::All => match other {
                SpecifiedBy::All => Some(Ordering::Equal),
                _ => Some(Ordering::Less),
            },
            SpecifiedBy::Tag => match other {
                SpecifiedBy::All => Some(Ordering::Greater),
                SpecifiedBy::Tag => Some(Ordering::Equal),
                _ => Some(Ordering::Less),
            },
            SpecifiedBy::Parent(parent) => match other {
                SpecifiedBy::Name => Some(Ordering::Less),
                SpecifiedBy::Parent(other_parent) => {
                    Some(parent.chars().count().cmp(&other_parent.chars().count()))
                }
                _ => Some(Ordering::Greater),
            },
            SpecifiedBy::Name => match other {
                SpecifiedBy::Name => Some(Ordering::Equal),
                _ => Some(Ordering::Greater),
            },
        }
    }
}
