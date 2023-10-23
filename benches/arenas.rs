use std::time::Duration;

use bumpalo::Bump;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use id_arena::{Arena, Id};
use space_rs::{DirectoryItemType, Size};

pub struct StdItem {
    pub path_segment: String,
    pub item_type: DirectoryItemType,
    pub size_in_bytes: Size,
    pub child_count: usize,
    pub children: Vec<StdItem>,
}

pub struct BumpaloItem<'a> {
    pub path_segment: &'a str,
    pub item_type: DirectoryItemType,
    pub size_in_bytes: &'a Size,
    pub child_count: usize,
    pub children: Vec<BumpaloItem<'a>>,
}

pub struct IdArenaItem {
    pub parent: Option<Id<IdArenaItem>>,
    pub path_segment: String,
    pub item_type: DirectoryItemType,
    pub size_in_bytes: Size,
    pub child_count: usize,
    pub children: Vec<Id<IdArenaItem>>,
}

pub fn bench_arenas(c: &mut Criterion) {
    let mut group = c.benchmark_group("arenas");
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("std", |b| {
        let mut items = vec![];
        b.iter(|| {
            black_box({
                let mut parent = StdItem {
                    path_segment: "parent".into(),
                    item_type: DirectoryItemType::Directory,
                    size_in_bytes: Size::new(1234),
                    child_count: 1,
                    children: vec![],
                };

                parent.children.push(StdItem {
                    path_segment: "child".into(),
                    item_type: DirectoryItemType::File,
                    size_in_bytes: Size::new(1234),
                    child_count: 0,
                    children: vec![],
                });

                items.push(parent);
            })
        })
    });
    group.bench_function("bumpalo", |b| {
        let arena = Bump::new();
        b.iter(|| {
            black_box({
                let parent = arena.alloc(BumpaloItem {
                    path_segment: arena.alloc_str("parent"),
                    item_type: DirectoryItemType::Directory,
                    size_in_bytes: arena.alloc(Size::new(1234)),
                    child_count: 1,
                    children: vec![],
                });

                let child = {
                    BumpaloItem {
                        path_segment: arena.alloc_str("child"),
                        item_type: DirectoryItemType::File,
                        size_in_bytes: arena.alloc(Size::new(1234)),
                        child_count: 0,
                        children: vec![],
                    }
                };

                parent.children.push(child);
            })
        })
    });
    group.bench_function("id-arena", |b| {
        let mut arena = Arena::<IdArenaItem>::new();
        b.iter(|| {
            black_box({
                let parent = arena.alloc(IdArenaItem {
                    parent: None,
                    path_segment: "parent".to_string(),
                    item_type: DirectoryItemType::Directory,
                    size_in_bytes: Size::new(1234),
                    child_count: 1,
                    children: vec![],
                });

                let child = arena.alloc(IdArenaItem {
                    parent: Some(parent),
                    path_segment: "child".to_string(),
                    item_type: DirectoryItemType::File,
                    size_in_bytes: Size::new(1234),
                    child_count: 0,
                    children: vec![],
                });

                arena.get_mut(parent).unwrap().children.push(child);
            })
        })
    });
    group.finish();
}

criterion_group!(benches, bench_arenas);
criterion_main!(benches);
