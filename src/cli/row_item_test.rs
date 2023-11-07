use crate::cli::row_item::{RowItem, RowItemType};
use space_rs::Size;
use std::{
    cell::RefCell,
    path::{self, PathBuf},
    rc::Rc,
};

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
        path_segment: "/some/path".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        row_index: 0,
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
        path_segment: format!("some{}path", path::MAIN_SEPARATOR_STR),
        children: vec![],
        parent: None,
        descendant_count: 2,
        row_index: 0,
    }));
    let item2 = Rc::new(RefCell::new(RowItem {
        size: Size::new(1024),
        has_children: true,
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 0.1f32,
        path_segment: "to".to_string(),
        children: vec![],
        parent: Some(Rc::downgrade(&item1)),
        descendant_count: 1,
        row_index: 0,
    }));
    let item3 = Rc::new(RefCell::new(RowItem {
        size: Size::new(1024),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0.1f32,
        path_segment: "file".to_string(),
        children: vec![],
        parent: Some(Rc::downgrade(&item2)),
        descendant_count: 0,
        row_index: 0,
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
