use rust_car::{
    reader::{self, CarReader}, utils::extract_ipld_to_current_path,
};

/// Cat the file in car file by file id
/// e.g. ```cargo run --example extract```
/// the example extract files from carv1-basic.car
fn main() {
    let file = std::path::Path::new("test");
    let file = file.join("carv1-basic.car");
    let file = std::fs::File::open(file).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    let roots = reader.header().roots();
    for r in roots.iter() {
        extract_ipld_to_current_path(&mut reader, *r).unwrap();
    }
}
