use cid::{Cid};
use rust_car::{reader::{CarReader, self}, unixfs::UnixFs, error::CarError, Ipld};

fn cat_ipld(reader: &mut impl CarReader, file_cid: Cid) {
    let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
    match file_ipld {
        rust_car::Ipld::Bytes(b) => {
            let content = unsafe{std::str::from_utf8_unchecked(&b[..])};
            print!("{content}");
        }
        m @ rust_car::Ipld::Map(_) => {
            let unix_fs: Result<UnixFs, CarError> = (file_cid, m).try_into();
            let _ = unix_fs.map(|ufs| {
                ufs.children().iter().for_each(|cufs| {
                    cat_ipld(reader, cufs.cid().unwrap());
                });
            });
        }
        _ => {}
    }
}

/// Cat the file in car file by file id
/// e.g. ```cargo run --example cat_file bafkreiabltrd5zm73pvi7plq25pef3hm7jxhbi3kv4hapegrkfpkqtkbme```
/// the example cat file with cid in carv1-basic.car
fn main() {
    let cid = std::env::args().nth(1).expect("use cid as argument");
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    let file_cid = Cid::try_from(cid.as_str()).expect("cid format error");
    for r in roots.iter() {
        let root_ipld = reader.ipld(r).unwrap();
        let root: Result<UnixFs, CarError> = root_ipld.try_into();
        let root_dir = root.unwrap();
        let count = root_dir.children()
            .iter()
            .filter(|u| u.cid().unwrap() == file_cid)
            .count();
        if count > 0 {
            cat_ipld(&mut reader, file_cid);
        }
    }
}