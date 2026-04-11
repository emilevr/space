use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};

fn make_file_item(path_segment: &str, size: u64) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size),
        descendant_count: 0,
        children: vec![],
    }
}

fn make_empty_root(path_segment: &str) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::default(),
        descendant_count: 0,
        children: vec![],
    }
}
#[test]
fn sort_root_children_sorts_by_size_descending() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("small", 100));
    view_state.add_scanned_child(make_file_item("large", 5000));
    view_state.add_scanned_child(make_file_item("medium", 1000));

    view_state.sort_root_children();

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["large", "medium", "small"], names);
}

#[test]
fn sort_root_children_with_equal_sizes_sorts_by_name_ascending() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("zebra", 500));
    view_state.add_scanned_child(make_file_item("apple", 500));
    view_state.add_scanned_child(make_file_item("mango", 500));

    view_state.sort_root_children();

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["apple", "mango", "zebra"], names);
}

#[test]
fn add_scanned_child_updates_total_items_in_tree() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    assert_eq!(1, view_state.total_items_in_tree);

    view_state.add_scanned_child(make_file_item("child", 100));

    // Root + 1 child = 2 total items.
    assert_eq!(2, view_state.total_items_in_tree);
}

#[test]
fn add_scanned_child_with_descendants_increments_total_items_by_one_plus_descendant_count() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Child directory with 5 descendants already counted in its descendant_count.
    let deep_child = DirectoryItem {
        path_segment: "deep".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(6000),
        descendant_count: 5,
        children: vec![],
    };
    view_state.add_scanned_child(deep_child);

    // root(1) + deep(1) + deep's descendants(5) = 7
    assert_eq!(7, view_state.total_items_in_tree);
}

#[test]
fn add_scanned_child_empty_directory_increments_total_items_by_one() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Empty directory child (descendant_count == 0).
    let empty_dir = DirectoryItem {
        path_segment: "emptydir".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(0),
        descendant_count: 0,
        children: vec![],
    };
    view_state.add_scanned_child(empty_dir);

    // root(1) + emptydir(1) = 2
    assert_eq!(2, view_state.total_items_in_tree);
}

#[test]
fn add_scanned_child_sequential_additions_accumulate_total_items_correctly() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));
    assert_eq!(1, view_state.total_items_in_tree);

    // Add a plain file (descendant_count = 0).
    view_state.add_scanned_child(make_file_item("file.txt", 100));
    assert_eq!(2, view_state.total_items_in_tree);

    // Add a directory with 2 descendants.
    let dir_with_children = DirectoryItem {
        path_segment: "subdir".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(2000),
        descendant_count: 2,
        children: vec![],
    };
    view_state.add_scanned_child(dir_with_children);
    // 2 + 1 (subdir) + 2 (its descendants) = 5
    assert_eq!(5, view_state.total_items_in_tree);

    // Add another plain file.
    view_state.add_scanned_child(make_file_item("other.txt", 50));
    assert_eq!(6, view_state.total_items_in_tree);
}

#[test]
fn total_items_in_tree_accurate_after_all_children_added() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Add a mix of items with varying descendant counts.
    view_state.add_scanned_child(make_file_item("a.txt", 100));
    view_state.add_scanned_child(make_file_item("b.txt", 200));
    view_state.add_scanned_child(DirectoryItem {
        path_segment: "dir1".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(1000),
        descendant_count: 10,
        children: vec![],
    });
    view_state.add_scanned_child(DirectoryItem {
        path_segment: "dir2".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(500),
        descendant_count: 3,
        children: vec![],
    });

    // root(1) + a.txt(1) + b.txt(1) + dir1(1+10) + dir2(1+3) = 18
    assert_eq!(18, view_state.total_items_in_tree);
}
