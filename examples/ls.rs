use std::collections::{VecDeque, HashMap};

use cid::Cid;
use ipld::raw::RawCodec;
use rust_car::Ipld;
use rust_car::reader::{self, CarReader};
use rust_car::unixfs::UnixFs;

/// walk the node and print the files in the directory.
fn walk(vecq: &mut VecDeque<Cid>, reader: &mut impl CarReader) {
    let mut cache: HashMap<Cid, String> = HashMap::new();
    let raw_code: u64 = RawCodec.into();
    while let Some(file_cid) = vecq.pop_front() {
        let codec = file_cid.codec();
        let file_n = cache.get(&file_cid).map_or(file_cid.to_string(), |c| c.to_string());
        println!("{file_n}");
        if codec == raw_code {
            continue;
        }
        let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
        match file_ipld {
            m @ Ipld::Map(_) => {
                let unixfs: UnixFs = m.try_into().unwrap();
                for n in unixfs.children().into_iter() {
                    let cid = n.cid().unwrap();
                    
                    n.file_name().map(|f| {
                        if f.len() > 0 {
                            cache.insert(cid, file_n.clone() + "/" + f);
                            vecq.push_back(cid);
                        }
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
