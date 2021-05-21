use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub enum ConfigNet {
    None,
    Tap {
        address: Option<String>,
        gateway: Option<String>,
        nameservers: Option<Vec<String>>,
        tap: Option<String>,
    },
    User,
}

impl ConfigNet {
    pub fn updated(&self, other: &Self) -> ConfigNet {
        match (self, other) {
            (ConfigNet::None, _) => other.to_owned(),
            (
                ConfigNet::Tap {
                    address: self_address,
                    gateway: self_gateway,
                    nameservers: self_nameservers,
                    tap: self_tap,
                },
                ConfigNet::Tap {
                    address: other_address,
                    gateway: other_gateway,
                    nameservers: other_nameservers,
                    tap: other_tap,
                },
            ) => {
                let address = self_address.as_ref().or_else(|| other_address.as_ref()).cloned();
                let gateway = self_gateway.as_ref().or_else(|| other_gateway.as_ref()).cloned();
                let nameservers =
                    self_nameservers.as_ref().or_else(|| other_nameservers.as_ref()).cloned();
                let tap = self_tap.as_ref().or_else(|| other_tap.as_ref()).cloned();
                ConfigNet::Tap { address, gateway, nameservers, tap }
            }
            _ => self.to_owned(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ConfigNet::None)
    }

    pub fn is_tap(&self) -> bool {
        matches!(self, ConfigNet::Tap { .. })
    }

    pub fn is_user(&self) -> bool {
        matches!(self, ConfigNet::User)
    }
}

#[derive(Debug, Clone)]
pub enum Net {
    Tap {
        address: Option<String>,
        gateway: Option<String>,
        nameservers: Option<Vec<String>>,
        tap: String,
    },
    User,
}

impl Net {
    pub fn new(config: &ConfigNet) -> Result<Option<Net>> {
        match config {
            ConfigNet::None => Ok(None),
            ConfigNet::Tap { address, gateway, nameservers, tap } => {
                let address = address.to_owned();
                let gateway = gateway.to_owned();
                let nameservers = nameservers.to_owned();
                let tap = tap.to_owned().ok_or(Error::TapNetworkTapUnset)?;

                Ok(Some(Net::Tap { address, gateway, nameservers, tap }))
            }
            ConfigNet::User => Ok(Some(Net::User)),
        }
    }

    pub fn gateway4(&self) -> Option<String> {
        if let Net::Tap { gateway: Some(gateway), .. } = self {
            if gateway.parse::<Ipv4Addr>().is_ok() {
                return Some(gateway.to_string());
            }
        }

        None
    }

    pub fn gateway6(&self) -> Option<String> {
        if let Net::Tap { gateway: Some(gateway), .. } = self {
            if gateway.parse::<Ipv6Addr>().is_ok() {
                return Some(gateway.to_string());
            }
        }

        None
    }
}

struct Cidr {
    pub address: Option<String>,
    pub network: Option<String>,
}

impl Cidr {
    pub fn new<S: AsRef<str>>(cidr: S) -> Cidr {
        let cidr = cidr.as_ref();
        let cidr: Vec<&str> = cidr.split('/').collect();
        let (address, network) = match cidr.len() {
            1 => (Some(cidr[0].to_string()), None),
            2 => (Some(cidr[0].to_string()), Some(cidr[1].to_string())),
            _ => (None, None),
        };

        let address = if address.as_ref().and_then(|a| a.parse::<IpAddr>().ok()).is_some() {
            address
        } else {
            None
        };

        Cidr { address, network }
    }
}

pub fn address<S: AsRef<str>>(cidr: S) -> Option<String> {
    Cidr::new(cidr.as_ref()).address
}

pub fn is_cidr<S: AsRef<str>>(cidr: S) -> bool {
    let cidr = Cidr::new(cidr.as_ref());
    cidr.address.is_some() && cidr.network.is_some()
}
