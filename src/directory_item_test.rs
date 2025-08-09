use super::{DirectoryItem, DirectoryItemType};
use crate::{
    directory_item::{get_file_name_from_path, FILE_NAME_ERROR_VALUE},
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    Size,
};
use rstest::rstest;
use std::{
    cmp::Ordering,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};
use uuid::Uuid;

#[rstest]
#[case(123, 123, Ordering::Less)]
#[case(1, 2, Ordering::Greater)]
#[case(2, 1, Ordering::Less)]
fn cmp_with_given_size_in_bytes_returns_descending_ordering(
    #[case] size_in_bytes_1: u64,
    #[case] size_in_bytes_2: u64,
    #[case] expected_ordering: Ordering,
) {
    // Arrange
    let v1 = DirectoryItem {
        path_segment: "/1".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        descendant_count: 1,
        children: vec![DirectoryItem {
            path_segment: "1".to_string(),
            size_in_bytes: Size::new(size_in_bytes_1),
            children: vec![],
            descendant_count: 0,
            item_type: DirectoryItemType::File,
        }],
    };
    let v2 = DirectoryItem {
        path_segment: "/2".to_string(),
        size_in_bytes: Size::new(size_in_bytes_2),
        children: vec![],
        descendant_count: 0,
        item_type: DirectoryItemType::Directory,
    };

    // Act
    let ordering = v1.cmp(&v2);

    // Assert
    assert_eq!(expected_ordering, ordering);
}

#[rstest]
#[case(432, 432, Some(Ordering::Less))]
#[case(1, 7, Some(Ordering::Greater))]
#[case(99, 98, Some(Ordering::Less))]
fn partial_cmp_with_given_size_in_bytes_returns_descending_ordering(
    #[case] size_in_bytes_1: u64,
    #[case] size_in_bytes_2: u64,
    #[case] expected_ordering: Option<Ordering>,
) {
    // Arrange
    let v1 = DirectoryItem {
        path_segment: "/2".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        descendant_count: 1,
        children: vec![DirectoryItem {
            path_segment: "1".to_string(),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(size_in_bytes_1),
            descendant_count: 0,
            children: vec![],
        }],
    };
    let v2 = DirectoryItem {
        path_segment: "/3".to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size_in_bytes_2),
        descendant_count: 0,
        children: vec![],
    };

    // Act
    let ordering = v1.partial_cmp(&v2);

    // Assert
    println!(
        "v1 size = {} bytes, v2 size = {} bytes",
        v1.size_in_bytes, v2.size_in_bytes
    );
    assert_eq!(expected_ordering, ordering);
}

#[rstest]
#[case(777, 777, true)]
#[case(10, 20, false)]
#[case(20, 10, false)]
fn eq_with_given_size_in_bytes_returns_correct_result(
    #[case] size_in_bytes_1: u64,
    #[case] size_in_bytes_2: u64,
    #[case] expected_result: bool,
) {
    // Arrange
    let v1 = DirectoryItem {
        path_segment: "/3".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        descendant_count: 1,
        children: vec![DirectoryItem {
            path_segment: "1".to_string(),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(size_in_bytes_1),
            descendant_count: 0,
            children: vec![],
        }],
    };
    let v2 = DirectoryItem {
        path_segment: "/4".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_2),
        descendant_count: 0,
        children: vec![],
    };

    // Act
    let equal = v1.eq(&v2);

    // Assert
    println!(
        "v1 size = {} bytes, v2 size = {} bytes",
        v1.size_in_bytes, v2.size_in_bytes
    );
    assert_eq!(expected_result, equal);
}

#[rstest]
fn from_root_given_file_path_should_return_only_file_item() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let file_path = temp_dir.join("1").join("1.1");
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let item = DirectoryItem::from_root(&file_path, &should_exit);

    // Assert
    assert_eq!(file_path.display().to_string(), item.path_segment);
    assert_eq!(25000, item.size_in_bytes.get_value());
    assert_eq!(0, item.children.len());

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[rstest]
fn build_given_symbolic_link_dir_should_not_follow_link() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let file_paths = vec![temp_dir.join("1").join("1.11")];
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let items = DirectoryItem::build(file_paths, &should_exit);

    // Assert
    assert_eq!(1, items.len());
    assert_eq!(0, items[0].size_in_bytes.get_value());
    assert_eq!(0, items[0].children.len());

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[rstest]
fn debug_succeeds() {
    // Arrange
    let item = DirectoryItem {
        path_segment: "/1".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(777),
        descendant_count: 1,
        children: vec![DirectoryItem {
            path_segment: "2".to_string(),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(778),
            descendant_count: 0,
            children: vec![],
        }],
    };

    // Act
    let output = format!("{:?}", item);

    // Assert
    assert!(output.contains("/1"));
    assert!(output.contains('2'));
    assert!(output.contains("777"));
    assert!(output.contains("778"));
}

#[rstest]
fn from_root_given_non_existent_path_does_not_panic() {
    // Arrange
    let path = std::env::temp_dir().join(Uuid::new_v4().to_string());
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let item = DirectoryItem::from_root(&path, &should_exit);

    // Assert
    assert_eq!(path.display().to_string(), item.path_segment);
    assert_eq!(0, item.size_in_bytes.get_value());
    assert_eq!(0, item.children.len());
}

#[rstest]
fn get_child_items_given_non_existent_path_does_not_panic() {
    // Arrange
    let path = std::env::temp_dir().join(Uuid::new_v4().to_string());
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let children = DirectoryItem::get_child_items(&path, &should_exit);

    // Assert
    assert_eq!(1, children.len());
}

#[rstest]
fn get_fraction_given_total_size_in_bytes_of_0_should_return_0() {
    // Arrange
    let item = DirectoryItem {
        path_segment: "/1".to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(123),
        descendant_count: 0,
        children: vec![],
    };

    // Act
    let fraction = item.get_fraction(0);

    // Assert
    assert_eq!(0f32, fraction);
}

#[rstest]
#[case("/")]
fn get_file_name_from_path_with_root_dir_returns_error_value(#[case] path: &str) {
    // Arrange
    let path = PathBuf::from(path);

    // Act
    let name = get_file_name_from_path(&path);

    // Assert
    assert_eq!(FILE_NAME_ERROR_VALUE, name);
}

#[rstest]
#[case("/test", "test")]
#[case("/test/some.png", "some.png")]
fn get_file_name_with_non_root_dir_path_returns_last_segment(
    #[case] path: &str,
    #[case] segment: &str,
) {
    // Arrange
    let path = PathBuf::from(path);

    // Act
    let name = get_file_name_from_path(&path);

    // Assert
    assert_eq!(segment, name);
}
