use cid::Cid;
use ipld::{pb::DagPbCodec, prelude::Codec};

use crate::{error::CarError, Ipld, CarHeader, utils::{empty_pb_cid, pb_cid}};

mod writer_v1;
pub(crate) use writer_v1::CarWriterV1;

pub enum WriteStream<'bs> {
    Bytes(&'bs [u8]),
    End
}

pub trait CarWriter {
    fn write<T>(&mut self, cid: Cid, data: T) -> Result<(), CarError>
    where
        T: AsRef<[u8]>;

    fn write_stream<F, R>(&mut self, cid_f: F, stream_len: usize, r: &mut R) -> Result<Cid, CarError>
    where
        R: std::io::Read,
        F: FnMut(WriteStream) -> Option<Result<Cid, CarError>>;

    fn write_ipld(&mut self, ipld: Ipld) -> Result<Cid, CarError> {
        match ipld {
            Ipld::Bytes(buf) => {
                let file_cid = crate::utils::raw_cid(&buf);
                self.write(file_cid, &buf)?;
                Ok(file_cid)
            },
            fs_ipld @ ipld::Ipld::Map(_) => {
                let bs: Vec<u8> = DagPbCodec
                    .encode(&fs_ipld)
                    .map_err(|e| CarError::Parsing(e.to_string()))?;
                let cid = pb_cid(&bs);
                self.write(cid, &bs)?;
                Ok(cid)
            },
            _ => Err(CarError::Parsing("Not support write ipld.".to_lowercase()))
        }
    }

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
