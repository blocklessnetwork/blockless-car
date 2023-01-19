use cid::Cid;
use integer_encoding::VarIntReader;

use crate::{reader::CarReader, error::CarError};
use std::io::Read;

pub(crate) struct CarReaderV1<R> {
    inner: R,
}

impl<R> CarReaderV1<R> 
where
    R: Read
{
    pub fn new(inner: R) -> Self {
        Self {
            inner 
        }
    }

    fn read_node(&mut self) -> Result<Option<(Cid, Vec<u8>)>, CarError> {
        let len: usize = match self.inner.read_varint() {
            Ok(i) => i,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                }
                return Err(CarError::IO(e))
            }
        };
        let mut data = vec![0u8; len];
        //TOOD: max length check.
        self.inner.read_exact(&mut data[..])
            .map_err(|e| CarError::IO(e))?;
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
    fn header(&self) -> &crate::header::CarHeader {
        todo!()
    }

    fn read_next_node(&mut self) -> Result<Option<(Cid, Vec<u8>)>, CarError> {
        self.read_node()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_read() {
        
    }
}