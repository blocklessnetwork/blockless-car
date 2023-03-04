use std::{
    collections::{HashMap, VecDeque},
    fs,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    codec::Encoder,
    error::CarError,
    header::CarHeaderV1,
    unixfs::{FileType, Link, UnixFs},
    writer::{CarWriter, CarWriterV1, WriteStream},
    CarHeader, Ipld,
};
use cid::{
    multihash::{
        Code, 
        MultihashDigest, 
        Hasher,
        Multihash,
        Blake2b256,
    },
    Cid,
};
use ipld::{pb::DagPbCodec, prelude::Codec, raw::RawCodec};
use path_absolutize::*;

use super::BLAKE2B256_CODEC;

type WalkPath = (Rc<PathBuf>, Option<usize>);

type WalkPathCache = HashMap<Rc<PathBuf>, UnixFs>;

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
    // ensure sufficient file block size for head, after the root cid generated using the content, fill back the head.
    let mut root_cid = Some(empty_pb_cid());
    let header = CarHeader::new_v1(vec![root_cid.unwrap()]);
    let mut writer = CarWriterV1::new(to_carfile, header);
    walk_dir(
        path,
        |(abs_path, parent_idx), path_map| -> Result<(), CarError> {
            let unixfs = path_map.get_mut(abs_path).unwrap();
            for link in unixfs.links.iter_mut() {
                match link.guess_type {
                    FileType::Directory => {}
                    FileType::File => {
                        //TODO: split file when file size is bigger than the max section size.
                        let filepath = abs_path.join(link.name_ref());
                        let mut file = fs::OpenOptions::new().read(true).open(filepath)?;
                        let mut hash_codec = Blake2b256::default();
                        let cid_gen = |w: WriteStream| {
                            match w {
                                WriteStream::Bytes(bs) => {
                                    hash_codec.update(bs);
                                    None
                                },
                                WriteStream::End => {
                                    let bs = hash_codec.finalize();
                                    let h = match Multihash::wrap(BLAKE2B256_CODEC, bs) {
                                        Ok(h) => h,
                                        Err(e) => return Some(Err(CarError::Parsing(e.to_string()))),
                                    };
                                    Some(Ok(Cid::new_v1(RawCodec.into(), h)))
                                },
                            }
                        };
                        let file_cid = writer.write_stream(cid_gen, link.tsize as usize, &mut file)?;
                        link.hash = file_cid;
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
            match abs_path.parent() {
                Some(parent) => {
                    let parent = Rc::new(parent.to_path_buf());

                    if let Some((p, pos)) = path_map.get_mut(&parent).zip(*parent_idx) {
                        p.links[pos].hash = cid;
                    }
                }
                None => unimplemented!("should not happend"),
            }
            Ok(())
        },
    )?;
    let root_cid = root_cid.ok_or(CarError::NotFound("root cid not found.".to_string()))?;
    let header = CarHeader::V1(CarHeaderV1::new(vec![root_cid]));
    writer.rewrite_header(header)
}

pub fn pipe_raw_cid<R, W>(r: &mut R, w: &mut W) -> Result<Cid, CarError> 
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut hash_codec = cid::multihash::Blake2b256::default();
    let mut bs = [0u8; 1024];
    while let Ok(n) = r.read(&mut bs) {
        hash_codec.update(&bs[0..n]);
        w.write_all(&bs[0..n])?;
    }
    let bs = hash_codec.finalize();
    let h = cid::multihash::Multihash::wrap(BLAKE2B256_CODEC, bs);
    let h = h.map_err(|e| CarError::Parsing(e.to_string()))?;
    Ok(Cid::new_v1(DagPbCodec.into(), h))
}

#[inline(always)]
pub fn empty_pb_cid() -> Cid {
    pb_cid(&[])
}

#[inline(always)]
pub fn pb_cid(data: &[u8]) -> Cid {
    let h = Code::Blake2b256.digest(data);
    Cid::new_v1(DagPbCodec.into(), h)
}

#[inline(always)]
pub fn raw_cid(data: &[u8]) -> Cid {
    let h = Code::Blake2b256.digest(data);
    Cid::new_v1(RawCodec.into(), h)
}

/// walk all directory, and record the directory informations.
/// `dir_queue` is the dir queue for hold the directory.
/// `WalkPath` contain the index in children.
fn walk_inner(
    dir_queue: &mut VecDeque<Rc<PathBuf>>,
    path_map: &mut WalkPathCache,
) -> Result<Vec<WalkPath>, CarError> {
    let mut dirs = Vec::new();
    while let Some(dir_path) = dir_queue.pop_back() {
        let mut unix_dir = UnixFs {
            file_type: FileType::Directory,
            ..Default::default()
        };
        for entry in fs::read_dir(&*dir_path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();
            let abs_path = file_path.absolutize()?.to_path_buf();

            let name = entry.file_name().to_str().unwrap_or("").to_string();
            let tsize = entry.metadata()?.len();
            let mut link = Link {
                name,
                tsize,
                ..Default::default()
            };
            if file_type.is_file() {
                link.guess_type = FileType::File;
                unix_dir.add_link(link);
            } else if file_type.is_dir() {
                let rc_abs_path = Rc::new(abs_path);
                link.guess_type = FileType::Directory;
                let idx = unix_dir.add_link(link);
                dirs.push((rc_abs_path.clone(), Some(idx)));
                dir_queue.push_back(rc_abs_path);
            }
            //skip other types.
        }
        path_map.insert(dir_path, unix_dir);
    }
    dirs.reverse();
    Ok(dirs)
}

pub fn walk_dir<T>(root: impl AsRef<Path>, mut walker: T) -> Result<(), CarError>
where
    T: FnMut(&WalkPath, &mut WalkPathCache) -> Result<(), CarError>,
{
    let src_path = root.as_ref().absolutize()?;
    let mut queue = VecDeque::new();
    let mut path_map: HashMap<Rc<PathBuf>, UnixFs> = HashMap::new();
    let root_path: Rc<PathBuf> = Rc::new(src_path.into());
    queue.push_back(root_path.clone());
    let mut keys = walk_inner(&mut queue, &mut path_map)?;
    keys.push((root_path, None));
    for key in keys.iter() {
        walker(key, &mut path_map)?;
    }
    Ok(())
}
