use std::collections::{VecDeque, HashMap};

use cid::Cid;
use rust_car::Ipld;
use rust_car::reader::{self, CarReader};
use rust_car::unixfs::UnixFs;

/// walk the node and print the files in the directory.
fn walk(vecq: &mut VecDeque<Cid>, reader: &mut impl CarReader) {
    let mut cache: HashMap<Cid, String> = HashMap::new();
    while let Some(file_cid) = vecq.pop_front() {
        let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
        let file_n = cache.get(&file_cid).map_or(file_cid.to_string(), |c| c.to_string());
        println!("{file_n}");
        match file_ipld {
            m @ Ipld::Map(_) => {
                let unixfs: UnixFs = m.try_into().unwrap();
                for n in unixfs.children().into_iter() {
                    let cid = n.cid().unwrap();
                    vecq.push_back(cid);
                    n.file_name().map(|f| {
                        cache.insert(cid, file_n.clone() + "/" + f);
                    });
                }
            }
            _ => {}
        }
    }
}

/// e.g. ```cargo run --example ls file```
/// the example list file infomation in carv1-basic.car file
fn main() {
    let file_name = std::env::args().nth(1);
    let path = file_name.as_ref().map(|f| f.into()).unwrap_or_else(|| {
        let file = std::path::Path::new(".");
        file.join("111.car")
    });
    let file = std::fs::File::open(path).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    let mut queue: VecDeque<Cid> = VecDeque::new();
    for r in roots.iter() {
        queue.push_front(*r);
        walk(&mut queue, &mut reader);
    }
}
