use rust_car::reader::{CarReader, self};
use rust_car::unixfs::UnixFs;
use rust_car::error::CarError;

/// walk the node and print the files in the directory.
fn walk(node: &UnixFs) {
    let cid = node.cid().map(String::from);
    let file_n = node.name().or(cid.as_ref().map(String::as_str));
    let file_s = node.file_size();
    let file_type = node.file_type();

    println!("cid: {cid:?} fileName: {file_n:?} fileSize: {file_s:?} fileType: {file_type:?}");
    for n in node.children().iter() {
        walk(n)
    }
}

fn main() {
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    assert_eq!(reader.sections().len(), 6);
    for r in roots.iter() {
        let unix_fs: Result<UnixFs, CarError> = reader.unixfs(r);
        let mut unix_fs = unix_fs.unwrap();
        unix_fs.set_name(Some((*r).into()));
        walk(&unix_fs);
    }
}