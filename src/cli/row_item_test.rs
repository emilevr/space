use crate::cli::row_item::{RowItem, RowItemType};
use space_rs::Size;
use std::{
    cell::RefCell,
    path::{self, PathBuf},
    rc::Rc,
};

fn make_leaf_row_item(size: u64) -> RowItem {
    RowItem {
        size: Size::new(size),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0f32,
        peer_fraction: 0.0,
        path_segment: "item".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }
}

#[test]
fn display_outputs_path_and_number_of_children() {
    // Arrange
    let item = RowItem {
        size: Size::default(),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 0.1f32,
        peer_fraction: 0.0,
        path_segment: "/some/path".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    };

    // Act
    let output = format!("{}", item);

    // Assert
    assert_eq!(
        format!(
            "{} with {} children",
            item.path_segment,
            item.children.len()
        ),
        output
    );
}

#[test]
fn get_path_returns_correct_path() {
    // Arrange
    let item1 = Rc::new(RefCell::new(RowItem {
        size: Size::new(1024),
        has_children: true,
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 0.1f32,
        peer_fraction: 0.0,
        path_segment: format!("some{}path", path::MAIN_SEPARATOR_STR),
        children: vec![],
        parent: None,
        descendant_count: 2,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }));
    let item2 = Rc::new(RefCell::new(RowItem {
        size: Size::new(1024),
        has_children: true,
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 0.1f32,
        peer_fraction: 0.0,
        path_segment: "to".to_string(),
        children: vec![],
        parent: Some(Rc::downgrade(&item1)),
        descendant_count: 1,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }));
    let item3 = Rc::new(RefCell::new(RowItem {
        size: Size::new(1024),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0.1f32,
        peer_fraction: 0.0,
        path_segment: "file".to_string(),
        children: vec![],
        parent: Some(Rc::downgrade(&item2)),
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }));
    {
        item1.borrow_mut().children.push(item2.clone());
        item2.borrow_mut().children.push(item3.clone());
    }

    // Act
    let path1 = item1.borrow().get_path();
    let path2 = item2.borrow().get_path();
    let path3 = item3.borrow().get_path();

    // Assert
    assert_eq!(
        PathBuf::from("some").join("path").display().to_string(),
        path1.display().to_string()
    );
    assert_eq!(
        PathBuf::from("some")
            .join("path")
            .join("to")
            .display()
            .to_string(),
        path2.display().to_string()
    );
    assert_eq!(
        PathBuf::from("some")
            .join("path")
            .join("to")
            .join("file")
            .display()
            .to_string(),
        path3.display().to_string()
    );
}

// ─── Tests for update_fraction ───────────────────────────────────────────────

#[test]
fn update_fraction_calculates_correct_fraction() {
    let mut item = make_leaf_row_item(1000);

    item.update_fraction(4000);

    assert!(
        (item.incl_fraction - 0.25f32).abs() < f32::EPSILON,
        "Expected 0.25, got {}",
        item.incl_fraction
    );
}

#[test]
fn update_fraction_with_zero_total_sets_fraction_to_zero() {
    let mut item = make_leaf_row_item(1000);
    item.incl_fraction = 0.5f32; // Start with a non-zero value.

    item.update_fraction(0);

    assert_eq!(0f32, item.incl_fraction);
}

#[test]
fn update_fraction_with_full_size_sets_fraction_to_one() {
    let mut item = make_leaf_row_item(1000);

    item.update_fraction(1000);

    assert!(
        (item.incl_fraction - 1.0f32).abs() < f32::EPSILON,
        "Expected 1.0, got {}",
        item.incl_fraction
    );
}

#[test]
fn update_fraction_recursively_updates_children() {
    let child = Rc::new(RefCell::new(make_leaf_row_item(500)));
    let mut parent = RowItem {
        size: Size::new(1000),
        has_children: true,
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 0f32,
        peer_fraction: 0.0,
        path_segment: "parent".to_string(),
        children: vec![child.clone()],
        parent: None,
        descendant_count: 1,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    };

    parent.update_fraction(2000);

    assert!(
        (parent.incl_fraction - 0.5f32).abs() < f32::EPSILON,
        "Expected parent fraction 0.5, got {}",
        parent.incl_fraction
    );
    assert!(
        (child.borrow().incl_fraction - 0.25f32).abs() < f32::EPSILON,
        "Expected child fraction 0.25, got {}",
        child.borrow().incl_fraction
    );
}
