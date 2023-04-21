use blockless_car::utils::archive_local;

/// Cat the file in car file by file id
/// e.g. ```cargo run --example `archive directory` `target car file`.```
fn main() {
    let file_name = std::env::args().nth(1).expect("use directory as argument");
    let target = std::env::args()
        .nth(2)
        .expect("need the target file as argument");
    let file = std::fs::File::create(target).unwrap();
    archive_local(file_name, file).unwrap();
}
