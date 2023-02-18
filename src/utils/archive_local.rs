use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    codec::Encoder,
    error::CarError,
    header::CarHeaderV1,
    unixfs::{FileType, UnixFs},
    writer::{CarWriter, CarWriterV1},
    CarHeader, Ipld,
};
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use ipld::{pb::DagPbCodec, prelude::Codec, raw::RawCodec};
use path_absolutize::*;

/// archive the directory to the target CAR format file
/// `path` is the directory archived in to the CAR file.
/// `to_carfile` is the target file.
pub fn archive_local<T>(path: impl AsRef<Path>, to_carfile: T) -> Result<(), CarError>
where
    T: std::io::Write + std::io::Seek,
{
    let src_path = path.as_ref();
    if !src_path.exists() {
        return Err(CarError::IO(io::ErrorKind::NotFound.into()));
    }
    let root_path = src_path.absolutize().unwrap();
    let path = root_path.to_path_buf();
    let mut root_cid = Some(pb_cid(b""));
    let header = CarHeader::V1(CarHeaderV1::new(vec![root_cid.unwrap()]));
    let mut writer = CarWriterV1::new(to_carfile, header);
    walk_dir(path, |abs_path, path_map| -> Result<(), CarError> {
        
        let unixfs = path_map.get_mut(abs_path).unwrap();
        for ufs in unixfs.children.iter_mut() {
            match ufs.file_type {
                FileType::Directory => {}
                FileType::File => {
                    //TODO: split file
                    let filepath = abs_path.join(ufs.file_name.as_ref().unwrap());
                    let mut file = fs::OpenOptions::new().read(true).open(filepath)?;

                    let mut buf = Vec::<u8>::new();
                    file.read_to_end(&mut buf)?;
                    let file_cid = raw_cid(&buf);
                    writer.write(file_cid, &buf)?;
                    ufs.cid = Some(file_cid);
                }
                _ => unreachable!("not support!"),
            }
        }
        let fs_ipld: Ipld = unixfs.encode()?;
        
        let bs = DagPbCodec
            .encode(&fs_ipld)
            .map_err(|e| CarError::Parsing(e.to_string()))?;
        let cid = pb_cid(&bs);

        if root_path.as_ref() == abs_path.as_ref() {
            root_cid = Some(cid);
        }
        writer.write(cid, bs)?;
        unixfs.cid = Some(cid);
        let file_name = abs_path.file_name().map(|f| f.to_str()).flatten();
        match abs_path.parent() {
            Some(parent) => {
                let parent = Rc::new(parent.to_path_buf());
                path_map.get_mut(&parent).map(|p| {
                    let mut dirs: Vec<_> = p
                        .children
                        .iter_mut()
                        .filter(|f| matches!(f.file_type, FileType::Directory))
                        .collect();
                    if let Ok(pos) = dirs.binary_search_by(|u| {
                        let filen = u.file_name.as_ref().map(String::as_str);
                        filen.cmp(&file_name)
                    }) {
                        dirs[pos].cid = Some(cid);
                    }
                });
            }
            None => unimplemented!("should not happend"),
        }
        Ok(())
    })?;
    let root_cid = root_cid.ok_or(CarError::NotFound("root cid not found.".to_string()))?;
    let header = CarHeader::V1(CarHeaderV1::new(vec![root_cid]));
    writer.rewrite_header(header)
}

#[inline(always)]
fn pb_cid(data: &[u8]) -> Cid {
    let h = Code::Blake2b256.digest(data);
    Cid::new_v1(DagPbCodec.into(), h)
}

#[inline(always)]
fn raw_cid(data: &[u8]) -> Cid {
    let h = Code::Blake2b256.digest(data);
    Cid::new_v1(RawCodec.into(), h)
}

fn walk_inner<'a>(
    dir_queue: &mut VecDeque<Rc<PathBuf>>,
    path_map: &'a mut HashMap<Rc<PathBuf>, UnixFs>,
) -> Result<Vec<Rc<PathBuf>>, CarError>
{
    let mut dirs = Vec::new();
    while dir_queue.len() > 0 {
        let dir_path = dir_queue.pop_back().unwrap();
        dirs.push(dir_path.clone());
        let mut unix_dir = UnixFs::default();
        unix_dir.file_type = FileType::Directory;
        for entry in fs::read_dir(&*dir_path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();
            let abs_path = file_path.absolutize()?.to_path_buf();

            let mut unixfile = UnixFs::default();
            unixfile.file_name = entry.file_name().to_str().map(String::from);
            unixfile.file_size = Some(entry.metadata()?.len());
            if file_type.is_file() {
                unixfile.file_type = FileType::File;
            }
            if file_type.is_dir() {
                unixfile.file_type = FileType::Directory;
                let rc_abs_path = Rc::new(abs_path);
                dir_queue.push_back(rc_abs_path);
            }
            unix_dir.add_child(unixfile);
            //skip other types.
        }
        path_map.insert(dir_path, unix_dir);
    }
    dirs.reverse();
    Ok(dirs)
}

pub fn walk_dir<T>(root: impl AsRef<Path>, mut walker: T) -> Result<(), CarError>
where
    T: FnMut(&Rc<PathBuf>, &mut HashMap<Rc<PathBuf>, UnixFs>) -> Result<(), CarError>,
{
    let src_path = root.as_ref().absolutize()?;
    let mut queue = VecDeque::new();
    let mut path_map: HashMap<Rc<PathBuf>, UnixFs> = HashMap::new();
    let root_path: Rc<PathBuf> = Rc::new(src_path.into());
    queue.push_back(root_path.clone());
    let keys = walk_inner(&mut queue, &mut path_map)?;
    for key in keys.iter() {
        walker(key, &mut path_map)?;
    }
    Ok(())
}

