mod error;
mod header;
mod reader;
mod writer;
mod section;
mod codec;
mod unixfs_codec;
mod unixfs;
mod pb;

pub use codec::{Decoder};
pub use header::CarHeader;
#[allow(unused)]
pub(crate) use reader::{CarReader, CarReaderV1};
#[allow(unused)]
pub(crate) use writer::{CarWriter, CarWriterV1};

pub enum IpldType {
    Null,
    /// Represents a boolean value.
    Bool,
    /// Represents an integer.
    Integer,
    /// Represents a floating point value.
    Float,
    /// Represents an UTF-8 string.
    String,
    /// Represents a sequence of bytes.
    Bytes,
    /// Represents a list.
    List,
    /// Represents a map of strings.
    Map,
    /// Represents a map of integers.
    Link,
}

pub struct Ipld(pub(crate) ipld::Ipld);

impl Ipld {
    pub fn ipld_type(&self) -> IpldType {
        match self.0 {
            ipld::Ipld::Null => IpldType::Null,
            ipld::Ipld::Bool(_) => IpldType::Bool,
            ipld::Ipld::Integer(_) => IpldType::Integer,
            ipld::Ipld::Float(_) => IpldType::Float,
            ipld::Ipld::String(_) => IpldType::String,
            ipld::Ipld::Bytes(_) => IpldType::Bytes,
            ipld::Ipld::List(_) => IpldType::List,
            ipld::Ipld::Map(_) => IpldType::Map,
            ipld::Ipld::Link(_) => IpldType::Link,
        }
    }

    #[inline]
    pub fn bool(&self) -> Option<bool> {
        match self.0 {
            ipld::Ipld::Bool(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn integer(&self) -> Option<i128> {
        match self.0 {
            ipld::Ipld::Integer(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn float(&self) -> Option<f64> {
        match self.0 {
            ipld::Ipld::Float(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn str(&self) -> Option<&str> {
        match self.0 {
            ipld::Ipld::String(ref v) => Some(v.as_str()),
            _ => None,
        }
    }

    #[inline]
    pub fn bytes(&self) -> Option<&[u8]> {
        match self.0 {
            ipld::Ipld::Bytes(ref v) => Some(&v[..]),
            _ => None,
        }
    }
}