use std::fs;
use std::path::Path;
use std::{path::PathBuf, fs::File};
use std::io::Write;

use cid::Cid;

use crate::error::CarError;
use crate::unixfs::{UnixFs, FileType};
use crate::{reader::CarReader, Ipld};


/// extract files to current path from CAR file. 
/// `cid` is the root cid
pub fn extract_ipld_to_current_path(
    reader: &mut impl CarReader,
    cid: Cid,
) -> Result<(), CarError> {
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
    extract_ipld_inner(reader, cid, parent, None)
}


/// inner function, extract files from CAR file. 
/// if the `parent` path is none, will use current path as root path.
/// `cid` is the file cid
fn extract_ipld_inner(
    reader: &mut impl CarReader,
    cid: Cid,
    parent: Option<PathBuf>,
    output: Option<&mut File>,
) -> Result<(), CarError> {
    let mut parent = parent;
    let file_ipld: Ipld = reader.ipld(&cid).unwrap();
    match file_ipld {
        Ipld::Bytes(b) => {
            output.map(|w| w.write_all(&b[..]));
        }
        m @ Ipld::Map(_) => {
            let unixfs: Result<UnixFs, CarError> = (cid, m).try_into();
            let unixfs = unixfs?;
            if parent.is_none() {
                parent = std::env::current_dir().ok();
            }
            match unixfs.file_type() {
                FileType::File => {
                    match output {
                        Some(o) => for cufs in unixfs.children().iter() {
                            // the file is contains by more than 1 block, the file name.
                            extract_ipld_inner(reader, cufs.cid().unwrap(), parent.clone(), Some(o))?;
                        },
                        None => for cufs in unixfs.children().iter() {
                            extract_ipld_inner(reader, cufs.cid().unwrap(), parent.clone(), None)?;
                        },
                    }
                }
                FileType::Directory => {
                    let dir_name = unixfs
                        .file_name()
                        .map(String::from)
                        .or(unixfs.cid().map(|cid| cid.to_string()));

                    let dir_name = dir_name.ok_or(CarError::InvalidFile("dir name is empty".into()))?;
                    parent = parent.map(|parent| parent.join(dir_name));
                    let parent = parent.unwrap();
                    fs::create_dir(&parent).map_err(|e| CarError::IO(e))?;
                    for cufs in unixfs.children().iter() {
                        let file_name = cufs.file_name();
                        let mut file_output = if let Some(f) = file_name {
                            Some(fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(parent.join(f))
                                .map_err(|e| CarError::IO(e))?)
                        } else {
                            None
                        };
                        extract_ipld_inner(
                            reader,
                            cufs.cid().unwrap(),
                            Some(parent.clone()),
                            file_output.as_mut(),
                        )?;
                    };
                }
                _ => unimplemented!("not implement"),
            };
        }
        _ => {}
    }
    Ok(())
}