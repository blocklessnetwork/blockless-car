
mod header_v1;
use header_v1::CarHeaderV1;
pub enum CarHeader {
    V1(CarHeaderV1),
    V2(),
}

impl CarHeader {
    
}