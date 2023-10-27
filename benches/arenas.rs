use bumpalo::Bump;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use id_arena::{Arena, Id};
use memory_stats::memory_stats;
use space_rs::{
    fast_arena::{FastId, FastIdArena},
    DirectoryItemType, Size, SizeDisplayFormat,
};
use std::time::Duration;

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

pub struct FastIdArenaItem {
    pub parent: Option<FastId<FastIdArenaItem>>,
    pub path_segment: String,
    pub item_type: DirectoryItemType,
    pub size_in_bytes: Size,
    pub child_count: usize,
    pub children: Vec<FastId<FastIdArenaItem>>,
}

fn bench_arenas(c: &mut Criterion) {
    let sample_size = 100;
    let mut group = c.benchmark_group("arenas");
    group.measurement_time(Duration::from_secs(3));
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(sample_size);

    benchmark_std_alloc(&mut group, sample_size);
    benchmark_id_arena(&mut group, sample_size);
    benchmark_fast_id_arena(&mut group, sample_size);
    benchmark_bumpalo(&mut group, sample_size);

    group.finish();
}

fn benchmark_std_alloc(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    sample_size: usize,
) {
    let mut sample_number: usize = 0;
    let mut start_physical_mem: usize = 0;
    let mut start_virtual_mem: usize = 0;

    group.bench_function("std", |b| {
        sample_number += 1;

        if sample_number == 1 {
            if let Some(usage) = memory_stats() {
                start_physical_mem = usage.physical_mem;
                start_virtual_mem = usage.virtual_mem;
            }
        }

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
        });

        if sample_number == sample_size {
            report_memory_usage("std", items.len(), start_physical_mem, start_virtual_mem);
        }
    });
}

fn benchmark_bumpalo(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    sample_size: usize,
) {
    let mut sample_number: usize = 0;
    let mut start_physical_mem: usize = 0;
    let mut start_virtual_mem: usize = 0;

    group.bench_function("bumpalo", |b| {
        sample_number += 1;

        if sample_number == 1 {
            if let Some(usage) = memory_stats() {
                start_physical_mem = usage.physical_mem;
                start_virtual_mem = usage.virtual_mem;
            }
        }

        let mut arena = Bump::new();
        let mut item_count: usize = 0;

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
                item_count += 1;
            })
        });

        if sample_number == sample_size {
            report_memory_usage("bumpalo", item_count, start_physical_mem, start_virtual_mem);
        }

        arena.reset();
    });
}

fn benchmark_id_arena(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    sample_size: usize,
) {
    let mut sample_number: usize = 0;
    let mut start_physical_mem: usize = 0;
    let mut start_virtual_mem: usize = 0;

    group.bench_function("id-arena", |b| {
        sample_number += 1;

        if sample_number == 1 {
            if let Some(usage) = memory_stats() {
                start_physical_mem = usage.physical_mem;
                start_virtual_mem = usage.virtual_mem;
            }
        }

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
        });

        if sample_number == sample_size {
            report_memory_usage(
                "id-arena",
                arena.len(),
                start_physical_mem,
                start_virtual_mem,
            );
        }
    });
}

fn benchmark_fast_id_arena(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    sample_size: usize,
) {
    let mut sample_number: usize = 0;
    let mut start_physical_mem: usize = 0;
    let mut start_virtual_mem: usize = 0;

    group.bench_function("fast-id-arena", |b| {
        sample_number += 1;

        if sample_number == 1 {
            if let Some(usage) = memory_stats() {
                start_physical_mem = usage.physical_mem;
                start_virtual_mem = usage.virtual_mem;
            }
        }

        let mut arena = FastIdArena::<FastIdArenaItem>::new();

        if sample_number == 1 {
            if let Some(usage) = memory_stats() {
                start_physical_mem = usage.physical_mem;
                start_virtual_mem = usage.virtual_mem;
            }
        }

        b.iter(|| {
            black_box({
                let parent = arena.alloc(FastIdArenaItem {
                    parent: None,
                    path_segment: "parent".to_string(),
                    item_type: DirectoryItemType::Directory,
                    size_in_bytes: Size::new(1234),
                    child_count: 1,
                    children: vec![],
                });

                let child = arena.alloc(FastIdArenaItem {
                    parent: Some(parent),
                    path_segment: "child".to_string(),
                    item_type: DirectoryItemType::File,
                    size_in_bytes: Size::new(1234),
                    child_count: 0,
                    children: vec![],
                });

                arena.get_mut(parent).unwrap().children.push(child);
            })
        });

        if sample_number == sample_size {
            report_memory_usage(
                "fast-id-arena",
                arena.len(),
                start_physical_mem,
                start_virtual_mem,
            );
        }
    });
}

fn report_memory_usage(
    arena_name: &str,
    item_count: usize,
    start_physical_mem: usize,
    start_virtual_mem: usize,
) {
    println!("\n[{}]", arena_name);

    println!("Allocated items          = {}", item_count);

    println!(
        "Physical memory at start = {}",
        Size::new(start_physical_mem as u64).to_string(SizeDisplayFormat::Binary)
    );
    println!(
        " Virtual memory at start = {}",
        Size::new(start_virtual_mem as u64).to_string(SizeDisplayFormat::Binary)
    );

    if let Some(usage) = memory_stats() {
        println!(
            "Physical memory at end   = {}",
            Size::new(usage.physical_mem as u64).to_string(SizeDisplayFormat::Binary)
        );
        println!(
            " Virtual memory at end   = {}",
            Size::new(usage.virtual_mem as u64).to_string(SizeDisplayFormat::Binary)
        );

        let physical_mem_delta = if usage.physical_mem > start_physical_mem {
            usage.physical_mem - start_physical_mem
        } else {
            0
        };
        println!(
            "Physical memory delta    = {}",
            Size::new(physical_mem_delta as u64).to_string(SizeDisplayFormat::Binary)
        );

        let virtual_mem_delta = if usage.virtual_mem > start_virtual_mem {
            usage.virtual_mem - start_virtual_mem
        } else {
            0
        };
        println!(
            " Virtual memory delta    = {}",
            Size::new(virtual_mem_delta as u64).to_string(SizeDisplayFormat::Binary)
        );
    } else {
        println!("Couldn't get the current memory usage!");
    }
}

criterion_group!(benches, bench_arenas);
criterion_main!(benches);
