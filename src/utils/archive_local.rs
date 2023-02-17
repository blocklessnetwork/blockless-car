use std::{path::{Path, PathBuf}, collections::{VecDeque, HashMap}, fs, io::{self, Read}};

use crate::{error::CarError, Ipld, unixfs::{UnixFs, FileType}, CarHeader, header::CarHeaderV1, writer::{CarWriterV1, CarWriter}, codec::Encoder};
use cid::{multihash::{Code, MultihashDigest}, Cid};
use ipld::{prelude::Codec, pb::DagPbCodec};
use path_absolutize::*;

/// archive the directory to the target CAR format file
/// `path` is the directory archived in to the CAR file.
/// `to_carfile` is the target file.
pub fn archive_local<T>(
    path: impl AsRef<Path>,
    to_carfile: T,
) -> Result<(), CarError>
where
    T: std::io::Write
{
    let src_path = path.as_ref();
    if !src_path.exists() {
        return Err(CarError::IO(io::ErrorKind::NotFound.into()));
    }
    let path = src_path.absolutize().unwrap();
    let path = path.to_path_buf();
    let root_cid = dir_cid(&path)?;
    let header = CarHeader::V1(CarHeaderV1::new(vec![root_cid]));
    let mut writer = CarWriterV1::new(to_carfile, header);
    walk_dir(path, |abs_path, unixfs| -> Result<(), CarError> {
        let cid = unixfs.cid;
        let fs_ipld: Ipld = unixfs.encode()?;
        let bs = DagPbCodec.encode(&fs_ipld)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        writer.write(cid.unwrap(), bs)?;
        for ufs in unixfs.children.iter() {
            match ufs.file_type {
                FileType::Directory => {},
                FileType::File => {
                    let filepath = abs_path.join(ufs.file_name.as_ref().unwrap());
                    let mut file = fs::OpenOptions::new()
                        .read(true)
                        .open(filepath)?;
                    let mut buf = Vec::<u8>::new();
                    file.read_to_end(&mut buf)?;
                    //TODO: split file
                    writer.write(ufs.cid.unwrap(), &buf)?;
                },
                _ => unreachable!("not support!"),
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn dir_cid(p: &PathBuf) -> Result<Cid, CarError> {
    let full_path = p.to_str();
    let h = Code::Sha2_256.digest(full_path.unwrap().as_bytes());
    Cid::new_v0(h).map_err(|e| CarError::Parsing(e.to_string()))
}

fn file_cid(p: &PathBuf) -> Result<Cid, CarError> {
    let full_path = p.to_str();
    let h = Code::Sha2_256.digest(full_path.unwrap().as_bytes());
    Ok(Cid::new_v1(0, h))
}

fn walk_inner<'a>(
    dir_queue: &mut VecDeque<PathBuf>,
    path_map: &'a mut HashMap<PathBuf, UnixFs>,
) -> Result<(), CarError> {
    while dir_queue.len() > 0 {
        let parent = dir_queue.pop_back().unwrap();
        let mut unix_dir = UnixFs::new(dir_cid(&parent)?);
        unix_dir.file_type = FileType::Directory;
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();
            let abs_path = file_path.absolutize()?.to_path_buf();
            
            let mut unixfile = UnixFs::default();
            unixfile.file_name = entry.file_name().to_str().map(String::from);
            unixfile.file_size = Some(entry.metadata()?.len());
            if file_type.is_file() {
                unixfile.cid = Some(file_cid(&abs_path)?);
                unixfile.file_type = FileType::File;
            }
            if file_type.is_dir() {
                unixfile.cid = Some(dir_cid(&abs_path)?);
                unixfile.file_type = FileType::Directory;
                dir_queue.push_back(abs_path);
            }
            unix_dir.add_child(unixfile);
            //skip other types.
        }
        path_map.insert(parent, unix_dir);
    }
    Ok(())
}

pub fn walk_dir<T>(
    root: impl AsRef<Path>, 
    mut walker: T,
) -> Result<Cid, CarError> 
where
    T: FnMut(PathBuf, UnixFs) -> Result<(), CarError>
{
    let src_path = root.as_ref().absolutize()?;
    let mut queue = VecDeque::new();
    let mut path_map: HashMap<PathBuf, UnixFs> = HashMap::new();
    let root_path: PathBuf = src_path.into();
    queue.push_back(root_path.clone());
    walk_inner(&mut queue, &mut path_map)?;
    let mut root = None;
    for (key, p) in path_map.into_iter() {
        if key == root_path {
            root = p.cid;
        }
        walker(key, p)?;
    }
    root.ok_or(CarError::NotFound("Not cid found".to_string()))
}

mod test {
    use super::*;

    #[test]
    fn test_walk_dir() {
        let current = std::env::current_dir().unwrap();
        let pwd = current.join(".");
        let _rootcid = walk_dir(pwd, |path, ufs| {
            let cid = ufs.cid;
            assert_eq!(dir_cid(&path).ok(), cid);
            assert_eq!(ufs.file_type, FileType::Directory);
            Ok(())
        });
        
    }    
}