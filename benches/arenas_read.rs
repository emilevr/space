use criterion::{criterion_group, criterion_main, Criterion};
use id_arena::{Arena, Id};
use space_rs::{
    rapid_arena::{RapId, RapIdArena},
    DirectoryItemType, Size,
};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

const ITEM_COUNT: usize = 100000;
const ITEM_READ_COUNT_PER_ITERATION: usize = 5;

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

pub struct RapIdArenaItem {
    pub parent: Option<RapId<RapIdArenaItem>>,
    pub path_segment: String,
    pub item_type: DirectoryItemType,
    pub size_in_bytes: Size,
    pub child_count: usize,
    pub children: Vec<RapId<RapIdArenaItem>>,
}

fn bench_arenas_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("arenas_read");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(100);

    benchmark_std(&mut group);
    benchmark_id_arena(&mut group);
    benchmark_rapid(&mut group);

    group.finish();
}

fn benchmark_std(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    let mut items = Vec::<StdItem>::with_capacity(ITEM_COUNT);
    // Alloc
    for _ in 0..ITEM_COUNT {
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
    }

    group.bench_function("std", |b| {
        b.iter(|| {
            for i in 0..ITEM_COUNT * ITEM_READ_COUNT_PER_ITERATION {
                let _ = &items[i % ITEM_COUNT];
            }
        });
    });
}

fn benchmark_id_arena(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    let mut arena = Arena::<IdArenaItem>::new();
    let mut ids = vec![];

    // Alloc
    for _ in 0..ITEM_COUNT {
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

        ids.push(parent);
    }

    group.bench_function("id-arena", |b| {
        b.iter(|| {
            for i in 0..ITEM_COUNT * ITEM_READ_COUNT_PER_ITERATION {
                let _ = &arena.get(ids[i % ITEM_COUNT]);
            }
        });
    });
}

fn benchmark_rapid(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    let mut arena = RapIdArena::<RapIdArenaItem>::new();
    let mut ids = vec![];

    // Alloc
    for _ in 0..ITEM_COUNT {
        let mut parent = arena.alloc(RapIdArenaItem {
            parent: None,
            path_segment: "parent".to_string(),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(1234),
            child_count: 1,
            children: vec![],
        });

        let child = arena.alloc(RapIdArenaItem {
            parent: Some(parent),
            path_segment: "child".to_string(),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(1234),
            child_count: 0,
            children: vec![],
        });

        parent.deref_mut().children.push(child);

        ids.push(parent);
    }

    group.bench_function("rapid-arena", |b| {
        b.iter(|| {
            for i in 0..ITEM_COUNT * ITEM_READ_COUNT_PER_ITERATION {
                let _ = ids[i % ITEM_COUNT].deref();
            }
        });
    });
}

criterion_group!(benches, bench_arenas_read);
criterion_main!(benches);
