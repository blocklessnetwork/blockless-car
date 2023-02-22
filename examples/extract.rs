use rust_car::{
    reader::{self, CarReader},
    utils::extract_ipld_to_current_path,
};

/// extract all files in car file by file id
/// e.g. ```cargo run --example extract```
/// the example cat used file is carv1-basic.car
fn main() {
    let file_name = std::env::args().nth(1);
    let path = file_name.as_ref().map(|f| f.into()).unwrap_or_else(|| {
        let file = std::path::Path::new("test");
        file.join("carv1-basic.car")
    });
    let file = std::fs::File::open(path).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    for r in roots.iter() {
        extract_ipld_to_current_path(&mut reader, *r).unwrap();
    }
}
