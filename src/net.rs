use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub enum ConfigNet {
    None,
    Tap {
        address: Option<String>,
        gateway: Option<String>,
        mac: Option<String>,
        nameservers: Option<Vec<String>>,
        tap: Option<String>,
    },
    User,
}

impl ConfigNet {
    pub fn updated(&self, other: &Self) -> ConfigNet {
        match (self, other) {
            (
                ConfigNet::Tap {
                    address: self_address,
                    gateway: self_gateway,
                    mac: self_mac,
                    nameservers: self_nameservers,
                    tap: self_tap,
                },
                ConfigNet::Tap {
                    address: other_address,
                    gateway: other_gateway,
                    mac: other_mac,
                    nameservers: other_nameservers,
                    tap: other_tap,
                },
            ) => {
                let address = self_address.as_ref().or(other_address.as_ref()).cloned();
                let gateway = self_gateway.as_ref().or(other_gateway.as_ref()).cloned();
                let mac = self_mac.as_ref().or(other_mac.as_ref()).cloned();
                let nameservers =
                    self_nameservers.as_ref().or(other_nameservers.as_ref()).cloned();
                let tap = self_tap.as_ref().or(other_tap.as_ref()).cloned();
                ConfigNet::Tap { address, gateway, mac, nameservers, tap }
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
    None,
    Tap {
        address: Option<String>,
        gateway: Option<String>,
        mac: Option<String>,
        nameservers: Option<Vec<String>>,
        tap: String,
    },
    User,
}

impl Net {
    pub fn new(config: &Option<ConfigNet>) -> Result<Option<Net>> {
        match config {
            Some(ConfigNet::Tap { address, gateway, mac, nameservers, tap }) => {
                let address = address.to_owned();
                let gateway = gateway.to_owned();
                let mac = mac.to_owned();
                let nameservers = nameservers.to_owned();
                let tap = tap.to_owned().ok_or(Error::TapNetworkTapUnset)?;

                Ok(Some(Net::Tap { address, gateway, mac, nameservers, tap }))
            }
            Some(ConfigNet::None) => Ok(Some(Net::None)),
            Some(ConfigNet::User) => Ok(Some(Net::User)),
            None => Ok(None),
        }
    }

    pub fn gateway4(&self) -> Option<String> {
        if let Net::Tap { gateway: Some(gateway), .. } = self
            && gateway.parse::<Ipv4Addr>().is_ok()
        {
            return Some(gateway.to_string());
        }

        None
    }

    pub fn gateway6(&self) -> Option<String> {
        if let Net::Tap { gateway: Some(gateway), .. } = self
            && gateway.parse::<Ipv6Addr>().is_ok()
        {
            return Some(gateway.to_string());
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
