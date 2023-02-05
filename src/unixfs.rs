use cid::Cid;

use crate::pb::{unixfs::{mod_Data::DataType, Data}, self};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    Raw = 0,
    Directory = 1,
    File = 2,
    Metadata = 3,
    Symlink = 4,
    HAMTShard = 5,
}

impl From<DataType> for FileType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Raw => FileType::Raw,
            DataType::Directory => FileType::Directory,
            DataType::File => FileType::File,
            DataType::Metadata => FileType::Metadata,
            DataType::Symlink => FileType::Symlink,
            DataType::HAMTShard => FileType::HAMTShard,
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct UnixTime {
    pub seconds: i64,
    pub fractional_nanoseconds: Option<u32>,
}

impl From<pb::unixfs::UnixTime> for UnixTime {
    fn from(value: pb::unixfs::UnixTime) -> Self {
        Self {
            seconds: value.Seconds,
            fractional_nanoseconds: value.FractionalNanoseconds,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnixFs {
    pub file_type: FileType,
    pub file_size: Option<u64>,
    pub block_sizes: Vec<u64>,
    pub hash_type: Option<u64>,
    pub fanout: Option<u64>,
    pub mode: Option<u32>,
    pub mtime: Option<UnixTime>,
}

impl<'a> From<Data<'a>> for UnixFs {
    fn from(value: Data<'a>) -> Self {
        Self {
            file_type: value.Type.into(),
            file_size: value.filesize,
            block_sizes: value.blocksizes,
            hash_type: value.hashType,
            fanout: value.fanout,
            mode: value.mode,
            mtime: value.mtime.map(|t| t.into()),
        }
    }
}

pub struct IpldUnixFs {
    cid: Cid,
    unix_fs: UnixFs,
    name: String,
    children: Vec<IpldUnixFs>,
}

impl IpldUnixFs {

    pub fn new(cid: Cid, name: String, unix_fs: UnixFs) -> Self {
        Self {
            cid,
            name,
            unix_fs,
            children: Vec::new()
        }
    }
    
}
