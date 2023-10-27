use rstest::rstest;

use super::*;

#[derive(Debug, PartialEq)]
struct Something {
    value1: usize,
    value2: String,
}

#[test]
fn new_creates_single_bucket_with_expected_bucket_size() {
    // Arrange
    let items_per_bucket =
        max(page_size::get(), page_size::get_granularity()) / size_of::<Something>();

    // Act
    let arena = FastIdArena::<Something>::new();

    // Assert
    assert_eq!(1, arena.buckets.len());
    assert_eq!(0, arena.bucket_index);
    assert_eq!(items_per_bucket, arena.items_per_bucket());
}

#[test]
fn default_creates_single_bucket_with_expected_bucket_size() {
    // Arrange
    let items_per_bucket =
        max(page_size::get(), page_size::get_granularity()) / size_of::<Something>();

    // Act
    let arena = FastIdArena::<Something>::default();

    // Assert
    assert_eq!(1, arena.buckets.len());
    assert_eq!(0, arena.bucket_index);
    assert_eq!(items_per_bucket, arena.items_per_bucket());
}

#[test]
fn new_with_bucket_size_creates_single_bucket_with_expected_bucket_size() {
    // Arrange
    let items_per_bucket = 100;

    // Act
    let arena = FastIdArena::<Something>::new_with_bucket_size(items_per_bucket);

    // Assert
    assert_eq!(1, arena.buckets.len());
    assert_eq!(0, arena.bucket_index);
    assert_eq!(items_per_bucket, arena.items_per_bucket());
}

#[test]
fn alloc_then_get_returns_expected_item() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let value1 = 123;
    let value2 = "abc".to_string();

    // Act
    let id = arena.alloc(Something {
        value1,
        value2: value2.clone(),
    });

    // Assert
    let actual = arena.get(id).expect("The ID is expected to be valid!");
    assert_eq!(value1, actual.value1);
    assert_eq!(value2, actual.value2);
}

#[test]
fn cloned_id_returns_expected_item() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let value1 = 1024;
    let value2 = "some string".to_string();
    let id = arena.alloc(Something {
        value1,
        value2: value2.clone(),
    });

    // Act
    let cloned_id = id.clone();

    // Assert
    let entry = arena
        .get(cloned_id)
        .expect("The ID is expected to be valid!");
    assert_eq!(value1, entry.value1);
    assert_eq!(value2, entry.value2);
}

#[test]
fn copied_id_returns_expected_item() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let value1 = 1024;
    let value2 = "some string".to_string();
    let id = arena.alloc(Something {
        value1,
        value2: value2.clone(),
    });

    // Act
    let copied_id = id;

    // Assert
    let entry = arena
        .get(copied_id)
        .expect("The ID is expected to be valid!");
    assert_eq!(value1, entry.value1);
    assert_eq!(value2, entry.value2);
}

#[rstest]
#[case(3, 0, 1)]
#[case(3, 1, 1)]
#[case(3, 2, 1)]
#[case(3, 3, 1)]
#[case(3, 4, 2)]
fn alloc_creates_correct_number_of_buckets(
    #[case] items_per_bucket: usize,
    #[case] alloc_count: usize,
    #[case] expected_bucket_count: usize,
) {
    // Arrange
    let mut arena = FastIdArena::<Something>::new_with_bucket_size(items_per_bucket);

    // Act
    for i in 0..alloc_count {
        arena.alloc(Something {
            value1: i,
            value2: format!("i = {}", i),
        });
    }

    // Assert
    assert_eq!(alloc_count, arena.len());
    assert_eq!(expected_bucket_count, arena.buckets.len());
}

#[test]
fn index_operator_returns_expected_item() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let value1 = 777;
    let value2 = "a string".to_string();

    // Act
    let id = arena.alloc(Something {
        value1,
        value2: value2.clone(),
    });

    // Assert
    let actual = &arena[id];
    assert_eq!(value1, actual.value1);
    assert_eq!(value2, actual.value2);
}

#[test]
fn get_item_in_second_bucket_returns_expected_item() {
    // Arrange
    let bucket_size = 5;
    let mut arena = FastIdArena::<Something>::new_with_bucket_size(bucket_size);
    let mut ids = vec![];

    for i in 0..=bucket_size {
        ids.push(arena.alloc(Something {
            value1: i,
            value2: format!("i = {}", i),
        }));
    }

    // Act
    let entry = arena.get(ids[5]).expect("The ID is expected to be valid!");

    // Assert
    assert_eq!(bucket_size, entry.value1);
    assert_eq!(format!("i = {}", bucket_size), entry.value2);
}

#[test]
fn get_with_invalid_id_returns_none() {
    // Arrange
    let arena = FastIdArena::<Something>::new();
    let id: FastId<Something> = FastId::<Something> {
        index: 123,
        _t: PhantomData,
    };

    // Act
    let value = arena.get(id);

    // Assert
    assert!(value.is_none());
}

#[test]
fn get_mut_with_invalid_id_returns_none() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let id: FastId<Something> = FastId::<Something> {
        index: 123,
        _t: PhantomData,
    };

    // Act
    let value = arena.get_mut(id);

    // Assert
    assert!(value.is_none());
}

#[test]
fn get_mut_then_modified_modifies_correct_entry() {
    // Arrange
    let mut arena = FastIdArena::<Something>::new();
    let mut ids = vec![];
    let value1 = 111;
    for i in 0..=2 {
        ids.push(arena.alloc(Something {
            value1: value1 + i * 111,
            value2: format!("i = {}", i),
        }));
    }

    // Act
    let mut something = arena.get_mut(ids[1]).expect("Expected ID to be valid!");
    something.value1 += 1;
    something.value2 = "world".to_string();

    // Assert
    let entry = arena.get(ids[1]).expect("Expected ID to be valid!");
    assert_eq!(223, entry.value1);
    assert_eq!("world", entry.value2);
}

#[rstest]
#[case(10, 0)]
#[case(10, 5)]
#[case(10, 71)]
fn len_with_allocs_returns_correct_length(
    #[case] items_per_bucket: usize,
    #[case] alloc_count: usize,
) {
    // Arrange
    let mut arena = FastIdArena::<Something>::new_with_bucket_size(items_per_bucket);
    for i in 0..alloc_count {
        arena.alloc(Something {
            value1: i,
            value2: format!("entry {}", i),
        });
    }

    // Act
    let len = arena.len();

    // Assert
    assert_eq!(alloc_count, len);
}

#[test]
fn is_empty_given_empty_arena_returns_true() {
    // Arrange
    let arena = FastIdArena::<Something>::new();

    // Act
    let is_empty = arena.is_empty();

    // Assert
    assert!(is_empty);
}

#[test]
fn is_empty_given_non_empty_arena_returns_false() {
    // Arrange
    let mut arena = FastIdArena::<usize>::new();
    arena.alloc(123);

    // Act
    let is_empty = arena.is_empty();

    // Assert
    assert!(!is_empty);
}

#[rstest]
#[case(5, 0)]
#[case(5, 503)]
fn reset_results_in_single_empty_bucket(
    #[case] items_per_bucket: usize,
    #[case] alloc_count: usize,
) {
    // Arrange
    let mut arena = FastIdArena::<Something>::new_with_bucket_size(items_per_bucket);
    for i in 0..alloc_count {
        arena.alloc(Something {
            value1: i,
            value2: format!("entry {}", i),
        });
    }

    // Act
    arena.reset();

    // Assert
    assert_eq!(0, arena.bucket_index);
    assert_eq!(1, arena.buckets.len());
    assert_eq!(items_per_bucket, arena.items_per_bucket);
}
