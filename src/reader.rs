use cid::Cid;

use crate::{header::CarHeader, error::CarError};
use integer_encoding::VarIntReader;

mod reader_v1;

const MaxAllowedSectionSize: usize = 32 << 20;

pub fn ld_read<R>(mut reader: R) -> Result<Option<Vec<u8>>, CarError> 
where
    R: std::io::Read
{
    let len: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarError::IO(e))
        }
    };
    if len > MaxAllowedSectionSize {
        return Err(CarError::TooLargeSection(len))
    }
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data[..])
        .map_err(|e| CarError::IO(e))?;
    Ok(Some(data))
}

trait CarReader {

    fn header(&self) -> &CarHeader;

    fn read_next_section(&mut self) -> Result<Option<(Cid, Vec<u8>)>,CarError>;
}