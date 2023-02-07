use quick_protobuf::{MessageRead, BytesReader};

use crate::{Decoder, error::CarError, pb::unixfs::Data, unixfs::UnixFs, Ipld};


impl Decoder<UnixFs> for Ipld {

    fn decode(&self) -> Result<UnixFs, CarError> {
        match self.0 {
            ipld::Ipld::Map(ref m) => {
                let mut unix_fs: UnixFs = if let Some(ipld::Ipld::Bytes(data)) = m.get("Data") {
                    let mut reader = BytesReader::from_bytes(&data);
                    Data::from_reader(&mut reader, &data)
                        .map(|d| d.into())
                        .map_err(|e| CarError::Parsing(e.to_string()))?
                } else {
                    return Err(CarError::Parsing("ipld format error".into()));
                };
                if let Some(ipld::Ipld::List(links)) = m.get("Links") {
                    links.iter().for_each(|l| {
                        match l {
                            ipld::Ipld::Map(ref m) => {
                                let cid = if let Some(ipld::Ipld::Link(cid)) = m.get("Hash") {
                                    cid.clone()
                                } else {
                                    return;
                                };
                                let name = if let Some(ipld::Ipld::String(name)) = m.get("Name") {
                                    Some(name.clone())
                                } else {
                                    None
                                };
                                let size = if let Some(ipld::Ipld::Integer(size)) = m.get("Tsize") {
                                    Some(*size as u64)
                                } else {
                                    None
                                };
                                let mut child = UnixFs::new(cid);
                                child.name = name;
                                child.file_size = size;
                                unix_fs.add_child(child);
                            }
                            _ => {}
                        }
                    });
                }
                Ok(unix_fs)
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