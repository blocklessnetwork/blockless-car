use cid::Cid;


use crate::{reader::CarReader, error::CarError, header::CarHeader};
use std::io::Read;

use super::ld_read;

pub(crate) struct CarReaderV1<R> {
    inner: R,
    header: CarHeader
}

impl<R> CarReaderV1<R> 
where
    R: Read
{
    pub fn new(mut inner: R) -> Result<Self, CarError> {
        let header = CarHeader::read_header(&mut inner)?;
        Ok(Self {
            inner,
            header,
        })
    }

    fn read_section(&mut self) -> Result<Option<(Cid, Vec<u8>)>, CarError> {
        let mut data = match ld_read(&mut self.inner) {
            Ok(Some(d)) => d,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut cursor = std::io::Cursor::new(&mut data);
        let cid = Cid::read_bytes(&mut cursor)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        let pos = cursor.position() as usize;
        Ok(Some((cid, data[pos..].to_vec())))
    }
    
}

impl<R> CarReader for CarReaderV1<R>
where
    R: Read
{
    fn header(&self) -> &CarHeader {
        &self.header
    }

    fn read_next_section(&mut self) -> Result<Option<(Cid, Vec<u8>)>, CarError> {
        self.read_section()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_read() {
        let file = std::path::Path::new("test");
        let file = file.join("carv1-basic.car");
        let file = std::fs::File::open(file).unwrap();
        let mut reader = CarReaderV1::new(file).unwrap();
        reader.read_next_section().unwrap();
    }
}