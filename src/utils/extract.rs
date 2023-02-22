use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use cid::Cid;

use crate::error::CarError;
use crate::unixfs::{FileType, UnixFs};
use crate::{reader::CarReader, Ipld};

/// extract files to current path from CAR file.
/// `cid` is the root cid
pub fn extract_ipld_to_current_path(reader: &mut impl CarReader, cid: Cid) -> Result<(), CarError> {
    extract_ipld(reader, cid, None::<PathBuf>)
}

/// extract files from CAR file.
/// if the `parent` path is none, will use current path as root path.
/// `cid` is the root cid
pub fn extract_ipld(
    reader: &mut impl CarReader,
    cid: Cid,
    parent: Option<impl AsRef<Path>>,
) -> Result<(), CarError> {
    let parent = parent.map(|p| p.as_ref().into());
    extract_ipld_inner(reader, cid, parent)
}

struct UnixfsCache {
    inner: UnixFs, 
    path: PathBuf,
}

struct IndexRelation {
    parent_cid: Cid,
    index: usize,
}

impl IndexRelation {
    fn full_path(rel: Option<&IndexRelation>, cache: &HashMap<Cid, UnixfsCache>) -> Option<PathBuf> {
        let filename = rel.map(|r| cache.get(&r.parent_cid)
            .map(|f| f.inner.links[r.index].name_ref())
        ).flatten();
        let parent_path = rel
            .map(|r| cache
                .get(&r.parent_cid)
                .map(|p| &p.path)
            ).flatten();
        parent_path.zip(filename).map(|(p, n)| {
            let path: PathBuf = p.into();
            path.join(n)
        })
    }
}

enum Type {
    Directory,
    File,
    FileLinks(UnixFs),
}


/// inner function, extract files from CAR file.
/// if the `parent` path is none, will use current path as root path.
/// `cid` is the file cid
fn extract_ipld_inner(
    reader: &mut impl CarReader,
    cid: Cid,
    parent: Option<PathBuf>,
) -> Result<(), CarError> {
    let mut queue = VecDeque::<Cid>::new();
    let mut unixfs_cache: HashMap<Cid, UnixfsCache> = Default::default();
    let mut relations: HashMap<Cid, IndexRelation> = Default::default();
    queue.push_back(cid);
    let root_path = match parent {
        Some(ref p) => {
            let cid_path: PathBuf = cid.to_string().into();
            p.join(cid_path)
        },
        None => cid.to_string().into(),
    };
    while let Some(cid) = queue.pop_front() {
        let rel = relations.get(&cid);
        let full_path = match IndexRelation::full_path(rel, &unixfs_cache) {
            Some(f) => f,
            None => root_path.clone(),
        };
        let file_ipld: Ipld = reader.ipld(&cid).unwrap();
        let file_links = match file_ipld {
            Ipld::Bytes(b) => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&full_path)
                    .unwrap();  
                file.write_all(&b).unwrap();
                Type::File
            }
            m @ Ipld::Map(_) => {
                let unixfs: UnixFs = (cid, m).try_into()?;
                match unixfs.file_type {
                    FileType::File => Type::FileLinks(unixfs),
                    _=> {
                        for (idx, link) in unixfs.links().iter().enumerate() {
                            let rel = IndexRelation {
                                parent_cid: cid, 
                                index: idx,
                            };
                            queue.push_back(link.hash);
                            relations.insert(link.hash, rel);
                        }
                        let rel = relations.get(&cid);
                        let path = IndexRelation::full_path(rel, &unixfs_cache)
                            .unwrap_or_else(|| cid.to_string().into());
                        unixfs_cache.insert(cid, UnixfsCache { 
                            inner: unixfs, 
                            path: path,
                        });
                        Type::Directory
                    }
                }
            }
            _ => unimplemented!("not implement"),
        };
        
        match file_links {
            Type::FileLinks(f) => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&full_path)
                    .unwrap();  
                for ufs in f.links() {
                    let file_ipld: Ipld = reader.ipld(&ufs.hash).unwrap();
                    match file_ipld {
                        Ipld::Bytes(b) => {
                            file.write_all(&b).unwrap();
                        }
                        _ => unreachable!("should not happend."),
                    }
                }
            }
            Type::Directory => fs::create_dir(&full_path)?,
            _ => {},
        }
    }
    Ok(())
}
