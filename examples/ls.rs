use rust_car::reader;

/// e.g. ```cargo run --example ls file```
/// the example list file infomation in carv1-basic.car file
fn main() {
    let file_name = std::env::args().nth(1);
    let path = file_name.as_ref().map(|f| f.into()).unwrap_or_else(|| {
        let file = std::path::Path::new("test");
        file.join("carv1-basic.car")
    });
    let file = std::fs::File::open(path).unwrap();
    let mut reader = reader::new_v1(file).unwrap();
    rust_car::utils::list(&mut reader);
}
