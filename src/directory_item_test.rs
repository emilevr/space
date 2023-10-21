use super::{DirectoryItem, DirectoryItemType};
use crate::{
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    Size,
};
use rstest::rstest;
use std::{cmp::Ordering, path::PathBuf, sync::Arc};
use uuid::Uuid;

#[rstest]
#[case(123, 123, Ordering::Equal)]
#[case(1, 2, Ordering::Less)]
#[case(2, 1, Ordering::Greater)]
fn cmp_with_given_size_in_bytes_returns_correct_ordering(
    #[case] size_in_bytes_1: u64,
    #[case] size_in_bytes_2: u64,
    #[case] expected_ordering: Ordering,
) {
    // Arrange
    let v1 = DirectoryItem {
        path: Arc::new(PathBuf::from("/1")),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        child_count: 1,
        children: vec![DirectoryItem {
            path: Arc::new(PathBuf::from("/1/1")),
            size_in_bytes: Size::new(size_in_bytes_1),
            children: vec![],
            child_count: 0,
            item_type: DirectoryItemType::File,
        }],
    };
    let v2 = DirectoryItem {
        path: Arc::new(PathBuf::from("/2")),
        size_in_bytes: Size::new(size_in_bytes_2),
        children: vec![],
        child_count: 0,
        item_type: DirectoryItemType::Directory,
    };

    // Act
    let ordering = v1.cmp(&v2);

    // Assert
    assert_eq!(expected_ordering, ordering);
}

#[rstest]
#[case(432, 432, Some(Ordering::Equal))]
#[case(1, 7, Some(Ordering::Less))]
#[case(99, 98, Some(Ordering::Greater))]
fn partial_cmp_with_given_size_in_bytes_returns_correct_ordering(
    #[case] size_in_bytes_1: u64,
    #[case] size_in_bytes_2: u64,
    #[case] expected_ordering: Option<Ordering>,
) {
    // Arrange
    let v1 = DirectoryItem {
        path: Arc::new(PathBuf::from("/2")),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        child_count: 1,
        children: vec![DirectoryItem {
            path: Arc::new(PathBuf::from("/2/1")),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(size_in_bytes_1),
            child_count: 0,
            children: vec![],
        }],
    };
    let v2 = DirectoryItem {
        path: Arc::new(PathBuf::from("/3")),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size_in_bytes_2),
        child_count: 0,
        children: vec![],
    };

    // Act
    let ordering = v1.partial_cmp(&v2);

    // Assert
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
        path: Arc::new(PathBuf::from("/3")),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_1),
        child_count: 1,
        children: vec![DirectoryItem {
            path: Arc::new(PathBuf::from("/3/1")),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(size_in_bytes_1),
            child_count: 0,
            children: vec![],
        }],
    };
    let v2 = DirectoryItem {
        path: Arc::new(PathBuf::from("/4")),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(size_in_bytes_2),
        child_count: 0,
        children: vec![],
    };

    // Act
    let equal = v1.eq(&v2);

    // Assert
    assert_eq!(expected_result, equal);
}

#[rstest]
fn from_root_given_file_path_should_return_only_file_item() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let file_path = temp_dir.join("1").join("1.1");
    let file_path = Arc::new(file_path);

    // Act
    let item = DirectoryItem::from_root(&file_path);

    // Assert
    assert_eq!(file_path, item.path);
    assert_eq!(25000, item.size_in_bytes.get_value());
    assert_eq!(0, item.children.len());

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[rstest]
fn build_given_symbolic_link_dir_should_not_follow_link() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let file_path = temp_dir.join("1").join("1.10");
    let file_paths = vec![Arc::new(file_path)];

    // Act
    let items = DirectoryItem::build(file_paths);

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
        path: Arc::new(PathBuf::from("/1")),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(777),
        child_count: 1,
        children: vec![DirectoryItem {
            path: Arc::new(PathBuf::from("/2/3")),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(778),
            child_count: 0,
            children: vec![],
        }],
    };

    // Act
    let output = format!("{:?}", item);

    // Assert
    assert!(output.contains("/1"));
    assert!(output.contains("/2/3"));
    assert!(output.contains("777"));
    assert!(output.contains("778"));
}

#[rstest]
fn from_root_given_non_existent_path_does_not_panic() {
    // Arrange
    let path = Arc::new(std::env::temp_dir().join(Uuid::new_v4().to_string()));

    // Act
    let item = DirectoryItem::from_root(&path);

    // Assert
    assert_eq!(path, item.path);
    assert_eq!(0, item.size_in_bytes.get_value());
    assert_eq!(0, item.children.len());
}

#[rstest]
fn get_child_items_given_non_existent_path_does_not_panic() {
    // Arrange
    let path = Arc::new(std::env::temp_dir().join(Uuid::new_v4().to_string()));

    // Act
    let children = DirectoryItem::get_child_items(&path);

    // Assert
    assert_eq!(0, children.len());
}

#[rstest]
fn get_fraction_given_total_size_in_bytes_of_0_should_return_0() {
    // Arrange
    let item = DirectoryItem {
        path: Arc::new(PathBuf::from("/1")),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(123),
        child_count: 0,
        children: vec![],
    };

    // Act
    let fraction = item.get_fraction(0);

    // Assert
    assert_eq!(0f32, fraction);
}
