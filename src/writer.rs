use cid::Cid;

use crate::{error::CarError, CarHeader, utils::empty_pb_cid};

mod writer_v1;
pub(crate) use writer_v1::CarWriterV1;

pub trait CarWriter {
    fn write<T>(&mut self, cid: Cid, data: T) -> Result<(), CarError>
    where
        T: AsRef<[u8]>;

    fn rewrite_header(&mut self, header: CarHeader) -> Result<(), CarError>;

    fn flush(&mut self) -> Result<(), CarError>;
}

pub fn new_v1<W>(inner: W, header: CarHeader) -> Result<impl CarWriter, CarError>
where
    W: std::io::Write + std::io::Seek,
{
    Ok(CarWriterV1::new(inner, header))
}

pub fn new_v1_default_roots<W>(inner: W) -> Result<impl CarWriter, CarError>
where
    W: std::io::Write + std::io::Seek,
{
    Ok(CarWriterV1::new(inner, CarHeader::new_v1(vec![empty_pb_cid()])))
}
