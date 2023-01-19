use cid::Cid;

use crate::error::CarError;

mod writer_v1;

trait CarWriter {
    fn write<T>(&mut self, cid: Cid, data: T) -> Result<(), CarError>
    where
        T: AsRef<u8>;
}
