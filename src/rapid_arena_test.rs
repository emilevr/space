use super::*;
use rstest::rstest;

#[derive(Debug, PartialEq)]
struct Something {
    some_value: usize,
    some_string: String,
}

fn alloc_items(arena: &mut RapIdArena<Something>, count: usize) -> Vec<RapId<Something>> {
    let mut ids = vec![];
    for i in 0..count {
        ids.push(arena.alloc(Something {
            some_value: i,
            some_string: format!("i = {}", i),
        }));
    }
    ids
}

#[test]
fn new_creates_single_bucket_with_expected_bucket_size() {
    // Arrange
    let items_per_bucket = DEFAULT_BUCKET_SIZE_IN_BYTES / size_of::<Something>();

    // Act
    let arena = RapIdArena::<Something>::new();

    // Assert
    assert_eq!(1, arena.buckets.len());
    assert_eq!(0, arena.bucket_index);
    assert_eq!(items_per_bucket, arena.items_per_bucket());
}

#[test]
fn default_creates_single_bucket_with_expected_bucket_size() {
    // Arrange
    let items_per_bucket = DEFAULT_BUCKET_SIZE_IN_BYTES / size_of::<Something>();

    // Act
    let arena = RapIdArena::<Something>::default();

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
    let arena = RapIdArena::<Something>::new_with_bucket_size(items_per_bucket);

    // Assert
    assert_eq!(1, arena.buckets.len());
    assert_eq!(0, arena.bucket_index);
    assert_eq!(items_per_bucket, arena.items_per_bucket());
}

#[test]
fn alloc_then_get_returns_expected_item() {
    // Arrange
    let mut arena = RapIdArena::<Something>::new();
    let v1 = 123;
    let s1 = "abc".to_string();

    // Act
    let id = arena.alloc(Something {
        some_value: v1,
        some_string: s1.clone(),
    });

    // Assert
    let actual = arena.get(id).expect("The ID is expected to be valid!");
    assert_eq!(v1, actual.some_value);
    assert_eq!(s1, actual.some_string);
}

#[test]
fn cloned_id_returns_expected_item() {
    // Arrange
    let mut arena = RapIdArena::<Something>::new();
    let v1 = 1024;
    let s1 = "some string".to_string();
    let id = arena.alloc(Something {
        some_value: v1,
        some_string: s1.clone(),
    });

    // Act
    let cloned_id = id.clone();

    // Assert
    let entry = arena
        .get(cloned_id)
        .expect("The ID is expected to be valid!");
    assert_eq!(v1, entry.some_value);
    assert_eq!(s1, entry.some_string);
}

#[test]
fn copied_id_returns_expected_item() {
    // Arrange
    let mut arena = RapIdArena::<Something>::new();
    let v1 = 1024;
    let s1 = "some string".to_string();
    let id = arena.alloc(Something {
        some_value: v1,
        some_string: s1.clone(),
    });

    // Act
    let copied_id = id;

    // Assert
    let entry = arena
        .get(copied_id)
        .expect("The ID is expected to be valid!");
    assert_eq!(v1, entry.some_value);
    assert_eq!(s1, entry.some_string);
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
    let mut arena = RapIdArena::<Something>::new_with_bucket_size(items_per_bucket);

    // Act
    alloc_items(&mut arena, alloc_count);

    // Assert
    assert_eq!(alloc_count, arena.len());
    assert_eq!(expected_bucket_count, arena.buckets.len());
}

#[test]
fn index_operator_returns_expected_item() {
    // Arrange
    let mut arena = RapIdArena::<Something>::new();
    let v1 = 777;
    let s1 = "a string".to_string();

    // Act
    let id = arena.alloc(Something {
        some_value: v1,
        some_string: s1.clone(),
    });

    // Assert
    let actual = &arena[id];
    assert_eq!(v1, actual.some_value);
    assert_eq!(s1, actual.some_string);
}

#[test]
fn get_given_multiple_buckets_returns_each_item() {
    // Arrange
    let bucket_size = 5;
    let mut arena = RapIdArena::<Something>::new_with_bucket_size(bucket_size);
    let count = (bucket_size as f32 * 3.5f32) as usize;
    let ids = alloc_items(&mut arena, count);

    for i in 0..count {
        // Act
        let entry = arena.get(ids[i]).expect("The ID is expected to be valid!");

        // Assert
        assert_eq!(i, entry.some_value);
        assert_eq!(format!("i = {}", i), entry.some_string);
    }
}

#[test]
fn get_with_invalid_id_returns_none() {
    // Arrange
    let arena = RapIdArena::<Something>::new();
    let id: RapId<Something> = RapId::<Something> {
        bucket_index: 321,
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
    let mut arena = RapIdArena::<Something>::new();
    let id: RapId<Something> = RapId::<Something> {
        bucket_index: 111,
        index: 222,
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
    let mut arena = RapIdArena::<Something>::new();
    let ids = alloc_items(&mut arena, 3);

    // Act
    let mut something = arena.get_mut(ids[1]).expect("Expected ID to be valid!");
    something.some_value += 1;
    something.some_string = "world".to_string();

    // Assert
    let entry = arena.get(ids[1]).expect("Expected ID to be valid!");
    assert_eq!(2, entry.some_value);
    assert_eq!("world", entry.some_string);
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
    let mut arena = RapIdArena::<Something>::new_with_bucket_size(items_per_bucket);
    alloc_items(&mut arena, alloc_count);

    // Act
    let len = arena.len();

    // Assert
    assert_eq!(alloc_count, len);
}

#[test]
fn is_empty_given_empty_arena_returns_true() {
    // Arrange
    let arena = RapIdArena::<Something>::new();

    // Act
    let is_empty = arena.is_empty();

    // Assert
    assert!(is_empty);
}

#[test]
fn is_empty_given_non_empty_arena_returns_false() {
    // Arrange
    let mut arena = RapIdArena::<usize>::new();
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
    let mut arena = RapIdArena::<Something>::new_with_bucket_size(items_per_bucket);
    alloc_items(&mut arena, alloc_count);

    // Act
    arena.reset();

    // Assert
    assert_eq!(0, arena.bucket_index);
    assert_eq!(1, arena.buckets.len());
    assert_eq!(items_per_bucket, arena.items_per_bucket);
}
