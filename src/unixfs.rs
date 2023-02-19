use std::fmt::Display;

use cid::Cid;

use crate::pb::{
    self,
    unixfs::{mod_Data::DataType, Data},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    Raw = 0,
    Directory = 1,
    File = 2,
    Metadata = 3,
    Symlink = 4,
    HAMTShard = 5,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type = match self {
            FileType::Raw => "raw",
            FileType::Directory => "directory",
            FileType::File => "file",
            FileType::Metadata => "metadata",
            FileType::Symlink => "symlink",
            FileType::HAMTShard => "hasmtshard",
        };
        write!(f, "{file_type}")
    }
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

impl From<FileType> for DataType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Raw => DataType::Raw,
            FileType::Directory => DataType::Directory,
            FileType::File => DataType::File,
            FileType::Metadata => DataType::Metadata,
            FileType::Symlink => DataType::Symlink,
            FileType::HAMTShard => DataType::HAMTShard,
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

impl From<UnixTime> for pb::unixfs::UnixTime {
    fn from(value: UnixTime) -> Self {
        Self {
            Seconds: value.seconds,
            FractionalNanoseconds: value.fractional_nanoseconds,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UnixFs {
    pub(crate) cid: Option<Cid>,
    pub(crate) mode: Option<u32>,
    pub(crate) file_type: FileType,
    pub(crate) fanout: Option<u64>,
    pub(crate) block_sizes: Vec<u64>,
    pub(crate) file_size: Option<u64>,
    pub(crate) hash_type: Option<u64>,
    pub(crate) children: Vec<UnixFs>,
    pub(crate) mtime: Option<UnixTime>,
    pub(crate) file_name: Option<String>,
}

impl<'a> From<Data<'a>> for UnixFs {
    fn from(value: Data<'a>) -> Self {
        Self {
            cid: None,
            file_name: None,
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
    pub fn add_child(&mut self, child: UnixFs) -> usize {
        let idx = self.children.len();
        self.children.push(child);
        idx
    }

    #[inline(always)]
    pub fn children(&self) -> Vec<&UnixFs> {
        self.children.iter().collect()
    }

    #[inline(always)]
    pub fn mtime(&self) -> Option<&UnixTime> {
        self.mtime.as_ref()
    }

    #[inline(always)]
    pub fn mode(&self) -> Option<u32> {
        self.mode
    }

    #[inline(always)]
    pub fn fanout(&self) -> Option<u64> {
        self.fanout
    }

    #[inline(always)]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_ref().map(String::as_str)
    }

    #[inline(always)]
    pub fn hash_type(&self) -> Option<u64> {
        self.hash_type
    }

    #[inline(always)]
    pub fn block_sizes(&self) -> Vec<u64> {
        self.block_sizes.clone()
    }

    #[inline(always)]
    pub fn file_size(&self) -> Option<u64> {
        self.file_size
    }

    #[inline(always)]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    #[inline(always)]
    pub fn cid(&self) -> Option<Cid> {
        self.cid
    }
}
