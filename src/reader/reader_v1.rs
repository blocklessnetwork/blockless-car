use crate::reader::CarReader;
use std::io::Read;

pub(crate) struct CarReaderV1<R> {
    inner: R,

}

impl<R> CarReaderV1<R> 
where
    R: Read
{
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner 
        }
    }
    
}
