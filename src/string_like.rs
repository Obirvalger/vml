use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum StringOrUint {
    S(String),
    I(u64),
}

impl fmt::Display for StringOrUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringOrUint::S(s) => write!(f, "{}", s),
            StringOrUint::I(i) => write!(f, "{}", i),
        }
    }
}
