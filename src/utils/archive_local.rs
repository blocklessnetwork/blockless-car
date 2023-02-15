use std::{path::{Path, PathBuf}, collections::{VecDeque, HashMap, BinaryHeap}, fs};

use crate::{error::CarError, Ipld, unixfs::{UnixFs, FileType}};
use cid::{multihash::{Code, MultihashDigest}, Cid};
use path_absolutize::*;

/// achive the directory to the target CAR format file
/// `path` is the directory archived in to the CAR file.
/// `to_carfile` is the target file.
pub fn achive_local<T>(
    path: impl AsRef<Path>,
    to_carfile: T,
) -> Result<(), CarError>
where
    T: std::io::Write
{
    let src_path = path.as_ref();
    if src_path.is_dir() {
        return Ok(());
    }
    
    Ok(())
}

fn new_unixfs(p: &PathBuf) -> Result<UnixFs, CarError> {
    let full_path = p.to_str();
    let h = Code::Sha2_256.digest(full_path.unwrap().as_bytes());
    let cid = Cid::new_v0(h).map_err(|e| CarError::Parsing(e.to_string()))?;
    Ok(UnixFs::new(cid))
}

fn walk_inner<'a>(
    dir_queue: &mut VecDeque<PathBuf>,
    path_map: &'a mut HashMap<PathBuf, UnixFs>,
) -> Result<(), CarError> {
    while dir_queue.len() > 0 {
        let parent = dir_queue.pop_back().unwrap();
        let mut unix_dir = new_unixfs(&parent)?;
        unix_dir.file_type = FileType::Directory;
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                dir_queue.push_back(entry.path().absolutize()?.to_path_buf());
                continue;
            }
            if file_type.is_file() {
                let file_path = entry.path();
                let mut unixfile = new_unixfs(&file_path)?;
                unixfile.file_name = entry.file_name().to_str().map(String::from);
                unixfile.file_size = Some(entry.metadata()?.len());
                unixfile.file_type = FileType::File;
                unix_dir.add_child(unixfile);
            }
            //skip other types.
        }
        path_map.insert(parent, unix_dir);
    }
    Ok(())
}

pub fn walk_dir(root: impl AsRef<Path>) -> Result<Ipld, CarError> {
    let src_path = root.as_ref().absolutize()?;
    let mut queue = VecDeque::new();
    let mut path_map: HashMap<PathBuf, UnixFs> = HashMap::new();
    let root_path: PathBuf = src_path.into();
    queue.push_back(root_path.clone());
    walk_inner(&mut queue, &mut path_map)?;
    let keys: BinaryHeap<_> = path_map.keys().map(|s| s.clone()).collect();
    let mut keys = keys.into_sorted_vec();
    keys.reverse();
    let mut root = None;
    for key in keys.iter() {
        let unixfs = path_map.remove(key).unwrap();
        let parent = key.parent().map(|parent| {
            path_map.get_mut(parent)
        }).flatten();
        if parent.is_none()  {
            root = Some(unixfs);
        } else {
            parent.unwrap().add_child(unixfs);
        }
    }
    root.unwrap().try_into()
}

mod test {
    use super::*;

    #[test]
    fn test_walk() {
        let current = std::env::current_dir().unwrap();
        let dir_ipld = walk_dir(current.join(".")).unwrap();
        assert!(matches!(dir_ipld, ipld::Ipld::Map(_)));
        let unixfs: Result<UnixFs, CarError> = dir_ipld.try_into();
        let unixfs = unixfs.unwrap();
        assert!(matches!(unixfs.file_type, FileType::Directory));
    }    
}