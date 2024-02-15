use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_rs::DirectoryItem;
use std::{
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

const DIRECTORY_PATH: &str = "./tmp.sample";

pub fn directory_item_build_x2(c: &mut Criterion) {
    let path = &Path::new(DIRECTORY_PATH).to_path_buf();

    use std::thread::available_parallelism;
    let thread_count = available_parallelism().unwrap().get() * 2;
    println!("Using {} Rayon threads", thread_count);

    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()
        .unwrap();

    let should_exit = Arc::new(AtomicBool::new(false));

    c.bench_function(
        &format!("DirectoryItem::build() x2 on {}", path.display()),
        |b| {
            b.iter(|| {
                black_box(DirectoryItem::build(vec![path.clone()], &should_exit));
            })
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = directory_item_build_x2
}
criterion_main!(benches);
