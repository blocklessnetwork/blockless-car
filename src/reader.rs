use cid::Cid;

use crate::{header::CarHeader, error::CarError};

mod reader_v1;

trait CarReader {
    
    fn header(&self) -> &CarHeader;

    fn read_next_block(&mut self) -> Result<Option<(Cid, Vec<u8>)>,CarError>;

}