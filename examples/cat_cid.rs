use cid::Cid;
use blockless_car::{
    error::CarError,
    reader::{self, CarReader},
    unixfs::UnixFs,
    utils::cat_ipld,
};

/// Cat the file in car file by file id
/// e.g. ```cargo run --example cat_file bafkreiabltrd5zm73pvi7plq25pef3hm7jxhbi3kv4hapegrkfpkqtkbme```
/// the example cat file with cid in carv1-basic.car
fn main() {
    // let cid = std::env::args().nth(1).expect("use cid as argument");
    let cid = "bafkreiaqv66m5nd6mwgkk7h5lwqnjzj54s4f7knmnrjhb7ylzqfg2vdo54";
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    let file_cid = Cid::try_from(cid).expect("cid format error");
    for r in roots.iter() {
        let root_ipld = reader.ipld(r).unwrap();
        let root: Result<UnixFs, CarError> = root_ipld.try_into();
        let root_dir = root.unwrap();
        let count = root_dir
            .links()
            .iter()
            .filter(|u| u.hash() == file_cid)
            .count();
        if count > 0 {
            cat_ipld(&mut reader, file_cid).unwrap();
        }
    }
}
