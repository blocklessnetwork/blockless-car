use crate::{error::CarError, header::CarHeader};

use super::CarWriter;
use integer_encoding::VarIntWriter;


struct CarWriterV1<W> {
    inner: W,
    header: CarHeader,
    is_header_written: bool,
}

impl<W> CarWriterV1<W> 
where
    W: std::io::Write
{
    pub fn new(inner: W, header: CarHeader) -> Self {
        Self {
            inner,
            header,
            is_header_written: false,
        }
    }
}

impl<W> CarWriter for CarWriterV1<W> 
where
    W: std::io::Write
{
    fn write<T>(&mut self, cid_data: cid::Cid, data: T) -> Result<(), CarError>
    where
        T: AsRef<[u8]> {
        if !self.is_header_written {
            let head = self.header.encode()?;
            self.inner.write_varint(head.len())?;
            self.inner.write_all(&head)?;
        }
        let mut cid_buff = Vec::new();
        cid_data.write_bytes(&mut cid_buff)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        let data = data.as_ref();
        let len = data.len() + cid_buff.len();
        self.inner.write_varint(len)?;
        self.inner.write_all(&cid_buff)?;
        self.inner.write_all(&data)?;
        Ok(())
    }
}