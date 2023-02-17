use rust_car::error::CarError;
use rust_car::reader::{self, CarReader};
use rust_car::unixfs::UnixFs;

/// walk the node and print the files in the directory.
fn walk(node: &UnixFs) {
    let cid = node.cid().map(String::from);
    let file_n = node.file_name().or(cid.as_ref().map(String::as_str));
    let file_s = node.file_size();
    let file_type = node.file_type();

    println!("cid: {cid:?} fileName: {file_n:?} fileSize: {file_s:?} fileType: {file_type:?}");
    for n in node.children().iter() {
        walk(n)
    }
}

/// e.g. ```cargo run --example ls file```
/// the example list file infomation in carv1-basic.car file
fn main() {
    let file_name = std::env::args().nth(1);
    let path = file_name.as_ref()
        .map(|f| f.into())
        .unwrap_or_else(|| {
            let file = std::path::Path::new(".");
            file.join("111.car")
        });
    let file = std::fs::File::open(path).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    assert_eq!(reader.sections().len(), 6);
    for r in roots.iter() {
        let unix_fs: Result<UnixFs, CarError> = reader.unixfs(r);
        let unix_fs = unix_fs.unwrap();
        walk(&unix_fs);
    }
}
