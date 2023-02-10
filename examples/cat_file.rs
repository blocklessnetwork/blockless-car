use rust_car::{
    reader::{self, CarReader}, 
    utils::cat_ipld,
};

/// Cat the file in car file by file id
/// e.g. ```cargo run --example cat_file file name.```
/// the example cat used file is carv1-basic.car
fn main() {
    let file_name = std::env::args().nth(1).expect("use filename as argument");
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let cid = reader.search_file_cid(&file_name).expect("search file error.");
    cat_ipld(&mut reader, cid).unwrap();
}