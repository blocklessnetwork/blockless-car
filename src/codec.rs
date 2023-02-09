use crate::error::CarError;

pub trait Decoder<T> {
    fn decode(&self) -> Result<T, CarError>;
}

pub trait Encoder<T> {
    fn encode(&self) -> Result<&[u8], CarError>;
}
