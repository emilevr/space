use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_rs::DirectoryItem;
use std::path::Path;

const DIRECTORY_PATH: &str = "./tmp.sample";

pub fn directory_item_build_x2(c: &mut Criterion) {
    let path = &Path::new(DIRECTORY_PATH).to_path_buf();

    use std::thread::available_parallelism;
    let default_parallelism_approx = available_parallelism().unwrap().get() * 2;

    rayon::ThreadPoolBuilder::new()
        .num_threads(default_parallelism_approx)
        .build_global()
        .unwrap();

    c.bench_function(
        &format!("DirectoryItem::build() x2 on {}", path.display()),
        |b| {
            b.iter(|| {
                black_box(DirectoryItem::build(vec![path.clone()]));
            })
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = directory_item_build_x2
}
criterion_main!(benches);
