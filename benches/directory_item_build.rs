use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_rs::DirectoryItem;
use std::path::Path;

const DIRECTORY_PATH: &str = "./tmp.sample";

pub fn directory_item_build(c: &mut Criterion) {
    let path = &Path::new(DIRECTORY_PATH).to_path_buf();
    c.bench_function(
        &format!("DirectoryItem::build() on {}", path.display()),
        |b| {
            b.iter(|| {
                black_box(DirectoryItem::build(vec![path.clone()]));
            })
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = directory_item_build
}
criterion_main!(benches);
