use std::collections::BTreeMap;

use cid::Cid;
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

use crate::{
    codec::Encoder,
    error::CarError,
    pb::unixfs::Data,
    unixfs::{FileType, UnixFs},
    Decoder, Ipld,
};

impl Decoder<UnixFs> for Ipld {
    fn decode(&self) -> Result<UnixFs, CarError> {
        match self {
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
                    links.iter().for_each(|l| match l {
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
                            child.file_name = name;
                            child.file_size = size;
                            unix_fs.add_child(child);
                        }
                        _ => {}
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

impl TryFrom<(Cid, Ipld)> for UnixFs {
    type Error = CarError;

    fn try_from(value: (Cid, Ipld)) -> Result<Self, Self::Error> {
        value.1.decode().map(|mut v| {
            v.cid = Some(value.0);
            v
        })
    }
}

fn convert_to_ipld(value: &UnixFs) -> Result<Ipld, CarError> {
    let mut map: BTreeMap<String, Ipld> = BTreeMap::new();
    map.insert("Hash".to_string(), Ipld::Link(value.cid.unwrap()));
    let file_name: Ipld = Ipld::String(
        value
            .file_name
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or(String::new()),
    );
    let tsize = Ipld::Integer(value.file_size.unwrap_or(0) as i128);
    map.insert("Name".to_string(), file_name);
    map.insert("Tsize".to_string(), tsize);
    Ok(Ipld::Map(map))
}

impl Encoder<Ipld> for UnixFs {
    fn encode(&self) -> Result<Ipld, CarError> {
        match self.file_type {
            FileType::Directory | FileType::File => {
                let mut map = BTreeMap::new();
                let mut data = Data::default();
                data.Type = self.file_type.into();
                data.fanout = self.fanout;
                data.blocksizes = self.block_sizes.clone();
                data.mode = self.mode;
                data.filesize = self.file_size;
                data.hashType = self.hash_type;
                data.mtime = self.mtime().map(|s| s.clone().into());
                let mut buf: Vec<u8> = Vec::new();
                let mut bw = Writer::new(&mut buf);
                data.write_message(&mut bw).map_err(|e| CarError::Parsing(e.to_string()))?;
                map.insert("Data".into(), Ipld::Bytes(buf));
                let mut children_ipld: Vec<Ipld> = Vec::new();
                for child in self.children.iter() {
                    children_ipld.push(convert_to_ipld(child)?);
                }
                map.insert("Links".to_string(), Ipld::List(children_ipld));
                Ok(Ipld::Map(map))
            }
            _ => Err(CarError::Parsing("Not support unixfs format".into())),
        }
    }
}

impl TryFrom<UnixFs> for Ipld {
    type Error = CarError;

    fn try_from(value: UnixFs) -> Result<Self, Self::Error> {
        value.encode()
    }
}
