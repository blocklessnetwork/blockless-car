mod header_v1;
use std::io;

pub(crate) use header_v1::CarHeaderV1;

use cid::Cid;
use ipld::prelude::Codec;
use ipld_cbor::DagCborCodec;

use crate::{error::CarError, reader::read_block};

#[derive(Clone, Debug)]
pub enum CarHeader {
    V1(CarHeaderV1),
    V2(),
}

impl CarHeader {
    pub fn new_v1(roots: Vec<Cid>) -> Self {
        CarHeader::V1(CarHeaderV1::new(roots))
    }

    pub fn roots(&self) -> Vec<Cid> {
        match self {
            &CarHeader::V1(ref v1) => v1.roots.clone(),
            &CarHeader::V2() => todo!(),
        }
    }

    pub fn read_header<R>(r: R) -> Result<CarHeader, CarError>
    where
        R: io::Read + io::Seek,
    {
        let data = match read_block(r) {
            Ok(Some(d)) => d,
            Ok(None) => return Err(CarError::Parsing("Invalid Header".into())),
            Err(e) => return Err(e),
        };
        let header = CarHeader::decode(&data[..])?;
        Ok(header)
    }

    pub fn decode(buf: &[u8]) -> Result<CarHeader, CarError> {
        let header: CarHeaderV1 = DagCborCodec
            .decode(buf)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        if header.roots.is_empty() {
            return Err(CarError::Parsing("car roots is empty".to_owned()));
        }
        if header.version != 1 {
            return Err(CarError::InvalidFile(
                "Now CAR version 1 is supported only".to_string(),
            ));
        }
        Ok(CarHeader::V1(header))
    }

    pub fn encode(&self) -> Result<Vec<u8>, CarError> {
        match self {
            &CarHeader::V1(ref v1) => {
                let data = DagCborCodec
                    .encode(v1)
                    .map_err(|e| CarError::Parsing(e.to_string()))?;
                Ok(data)
            }
            &CarHeader::V2() => todo!(),
        }
    }
}
