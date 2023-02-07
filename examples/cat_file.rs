use cid::{Cid};
use rust_car::{reader::{CarReader, self}, unixfs::UnixFs, error::CarError, Ipld};

/// Cat the file in car file by file id
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
            let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
            match file_ipld {
                rust_car::Ipld::Bytes(b) => {
                    println!("{}", unsafe{std::str::from_utf8_unchecked(&b[..])});
                }
                _ => {}
            }
        }
    }
}