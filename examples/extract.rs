use cid::Cid;
use std::{io::Write, fs::{self, File}, path::PathBuf};
use rust_car::{
    reader::{
        CarReader, self
    }, 
    unixfs::{UnixFs, FileType}, 
    error::CarError, 
    Ipld
};

fn walk_ipld(
    reader: &mut impl CarReader, 
    cid: Cid, 
    parent: Option<PathBuf>,
    output: Option<&mut File>,
) {
    let mut parent = parent;
    let file_ipld: Ipld = reader.ipld(&cid).unwrap();
    match file_ipld {
        rust_car::Ipld::Bytes(b) => {
            output.map(|w| w.write_all(&b[..]));
        }
        m @ rust_car::Ipld::Map(_) => {
            let unixfs: Result<UnixFs, CarError> = (cid, m).try_into();
            let unixfs = unixfs.expect("not unix file system");
            if parent.is_none() {
                parent = std::env::current_dir().ok();   
            }
            println!("{unixfs:?}");
            match unixfs.file_type() {
                FileType::File => {
                    match output {
                        Some(o) => unixfs.children().iter().for_each(|cufs| {
                            // the file is contains by more than 1 block, the file name.
                            walk_ipld(reader, cufs.cid().unwrap(), parent.clone(), Some(o));
                        }),
                        None => unixfs.children().iter().for_each(|cufs| {
                            walk_ipld(reader, cufs.cid().unwrap(), parent.clone(), None);
                        }),
                    }
                },
                FileType::Directory => {
                    let dir_name = unixfs.file_name()
                        .map(String::from)
                        .or(unixfs.cid().map(|cid| cid.to_string()));
                    
                    let dir_name = dir_name.expect("dir name is empty");
                    parent = parent.map(|parent| {
                        let parent = parent.join(dir_name);
                        fs::create_dir(&parent).expect("create dir error.");
                        parent
                    });
                    let parent = parent.unwrap();
                    unixfs.children().iter().for_each(|cufs| {
                        let file_name = cufs.file_name();
                        let mut file_output = file_name.map(|f| {
                            fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(parent.join(f))
                                .expect("create file error.")
                        });
                        walk_ipld(reader, cufs.cid().unwrap(), Some(parent.clone()), file_output.as_mut());
                    });
                },
                _ => unimplemented!("not implement"),
            };
        }
        _ => {}
    }
}

/// Cat the file in car file by file id
/// e.g. ```cargo run --example extract```
/// the example extract files from carv1-basic.car
fn main() {
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    for r in roots.iter() {
        walk_ipld(&mut reader, *r, None, None);
    }
}