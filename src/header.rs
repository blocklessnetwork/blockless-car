
mod header_v1;
use header_v1::CarHeaderV1;

use cid::Cid;

pub enum CarHeader {
    V1(CarHeaderV1),
    V2(),
}

impl CarHeader {
    pub fn new_v1(roots: Vec<Cid>) -> Self {
        CarHeader::V1(CarHeaderV1::new(roots))
    }
}