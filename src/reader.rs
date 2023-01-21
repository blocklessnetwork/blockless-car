use cid::Cid;

use crate::{header::CarHeader, error::CarError};
use integer_encoding::VarIntReader;

pub(crate) use reader_v1::CarReaderV1;

mod reader_v1;

const MAX_ALLOWED_SECTION_SIZE: usize = 32 << 20;

pub fn ld_read<R>(mut reader: R) -> Result<Option<Vec<u8>>, CarError> 
where
    R: std::io::Read
{
    let l: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarError::IO(e))
        }
    };
    if l > MAX_ALLOWED_SECTION_SIZE {
        return Err(CarError::TooLargeSection(l))
    }
    let mut data = vec![0u8; l];
    reader.read_exact(&mut data[..])
        .map_err(|e| CarError::IO(e))?;
    Ok(Some(data))
}

pub trait CarReader {

    fn header(&self) -> &CarHeader;

    fn read_next_section(&mut self) -> Result<Option<(Cid, Vec<u8>)>,CarError>;

}