use std::collections::{VecDeque, HashMap};

use cid::Cid;
use ipld::raw::RawCodec;

use crate::{reader::CarReader, Ipld, unixfs::{UnixFs, FileType}, error::CarError};

/// walk the node and print the files in the directory.
fn walk(vecq: &mut VecDeque<Cid>, reader: &mut impl CarReader) -> Result<(), CarError> {
    let mut cache: HashMap<Cid, String> = HashMap::new();
    let raw_code: u64 = RawCodec.into();
    while let Some(file_cid) = vecq.pop_front() {
        let codec = file_cid.codec();
        let file_n = cache.get(&file_cid).map_or(file_cid.to_string(), |c| c.to_string());
        println!("{file_n}");
        // if the codec is RawCodec, the block is the file content block, 
        // it don't conatian the file info. we don't need walk continue.
        if codec == raw_code {
            continue;
        }
        let file_ipld: Ipld = reader.ipld(&file_cid)?;
        if let m @  Ipld::Map(_) = file_ipld {
            let unixfs: UnixFs = m.try_into()?;
            match unixfs.file_type() {
                FileType::Directory => {},
                _=> continue,
            }
            for n in unixfs.links().into_iter() {
                let cid = n.hash();
                cache.insert(cid, file_n.clone() + "/" + n.name_ref());
                vecq.push_back(cid);
            }
        }
    }
    Ok(())
}

/// the list car file  by reader.
pub fn list(reader: &mut impl CarReader)  -> Result<(), CarError> {
    let roots = reader.header().roots();
    let mut queue: VecDeque<Cid> = VecDeque::new();
    for r in roots.iter() {
        queue.push_front(*r);
        walk(&mut queue, reader)?;
    }
    Ok(())
}