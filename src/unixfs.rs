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

impl Default for FileType {
    fn default() -> Self {
        FileType::Raw
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UnixFs {
    pub(crate) cid: Option<Cid>,
    pub(crate) file_type: FileType,
    pub(crate) file_size: Option<u64>,
    pub(crate) block_sizes: Vec<u64>,
    pub(crate) hash_type: Option<u64>,
    pub(crate) name: Option<String>,
    pub(crate) fanout: Option<u64>,
    pub(crate) mode: Option<u32>,
    pub(crate) mtime: Option<UnixTime>,
    pub(crate) children: Vec<UnixFs>
}

impl<'a> From<Data<'a>> for UnixFs {
    fn from(value: Data<'a>) -> Self {
        Self {
            cid: None,
            name: None,
            file_type: value.Type.into(),
            file_size: value.filesize,
            block_sizes: value.blocksizes,
            hash_type: value.hashType,
            fanout: value.fanout,
            mode: value.mode,
            mtime: value.mtime.map(|t| t.into()),
            children: Default::default(),
        }
    }
}

impl UnixFs {
    pub fn new(cid: Cid) -> Self {
        Self {
            cid: Some(cid),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn add_child(&mut self, child: UnixFs) {
        self.children.push(child);
    }
}
