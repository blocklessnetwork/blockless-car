use crate::{error::CarError, header::CarHeader};

use super::CarWriter;
use integer_encoding::VarIntWriter;

pub(crate) struct CarWriterV1<W> {
    inner: W,
    header: CarHeader,
    is_header_written: bool,
}

impl<W> CarWriterV1<W>
where
    W: std::io::Write,
{

    fn write_header(&mut self) -> Result<(), CarError>{
        let head = self.header.encode()?;
        self.inner.write_varint(head.len())?;
        self.inner.write_all(&head)?;
        self.is_header_written = true;
        Ok(())
    }

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
    W: std::io::Write,
{
    fn write<T>(&mut self, cid_data: cid::Cid, data: T) -> Result<(), CarError>
    where
        T: AsRef<[u8]>,
    {
        if !self.is_header_written {
            self.write_header()?;
        }
        let mut cid_buff: Vec<u8> = Vec::new();
        cid_data
            .write_bytes(&mut cid_buff)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        let data = data.as_ref();
        let sec_len = data.len() + cid_buff.len();
        self.inner.write_varint(sec_len)?;
        self.inner.write_all(&cid_buff[..])?;
        self.inner.write_all(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CarError> {
        self.inner.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use ipld_cbor::DagCborCodec;

    use crate::header::{CarHeader, CarHeaderV1};
    use crate::reader::{CarReader, CarReaderV1};

    use super::*;
    use cid::multihash::{Code::Blake2b256, MultihashDigest};
    use cid::Cid;

    #[test]
    fn test_writer_read_v1() {
        let digest_test = Blake2b256.digest(b"test");
        let cid_test1 = Cid::new_v1(DagCborCodec.into(), digest_test);
        let digest_test2 = Blake2b256.digest(b"test2");
        let cid_test2 = Cid::new_v1(DagCborCodec.into(), digest_test2);
        let header = CarHeader::V1(CarHeaderV1::new(vec![cid_test2]));
        let mut buffer = Vec::new();
        let mut writer = CarWriterV1::new(&mut buffer, header);
        writer.write(cid_test1, b"test1").unwrap();
        writer.write(cid_test2, b"test2").unwrap();
        writer.flush().unwrap();
        let mut reader = Cursor::new(&buffer);
        let mut car_reader = CarReaderV1::new(&mut reader).unwrap();
        assert_eq!(car_reader.header().roots(), car_reader.header().roots());
        let sec1 = car_reader.read_next_section().unwrap().unwrap();
        let sec2 = car_reader.read_next_section().unwrap().unwrap();
        let sec3 = car_reader.read_next_section().unwrap();
        assert_eq!(sec1.0, cid_test1);
        assert_eq!(sec2.0, cid_test2);
        assert_eq!(sec3, None);
    }
}
