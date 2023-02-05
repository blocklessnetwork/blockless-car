use quick_protobuf::{MessageRead, BytesReader};

use crate::{Decoder, error::CarError, pb::unixfs::Data, unixfs::UnixFs, Ipld};


impl Decoder<UnixFs> for Ipld {

    fn decode(&self) -> Result<UnixFs, CarError> {
        match self.0 {
            ipld::Ipld::Map(ref m) => {
                if let Some(ipld::Ipld::Bytes(data)) = m.get("Data") {
                    let mut reader = BytesReader::from_bytes(&data);
                    Data::from_reader(&mut reader, &data)
                        .map(|d| d.into())
                        .map_err(|e| CarError::Parsing(e.to_string()))
                } else {
                    return Err(CarError::Parsing("ipld format error".into()));
                }
            }
            _ => return Err(CarError::Parsing("Not unixfs format".into())),
        }
    }
    
}

impl TryFrom<Ipld> for UnixFs {
    type Error = CarError;

    fn try_from(value: Ipld) -> Result<Self, Self::Error> {
        value.decode()
    }
}