use crate::cli::row_item::{RowItem, RowItemType};
use space_rs::Size;
use std::{path::PathBuf, sync::Arc};

#[test]
fn display_outputs_path_and_number_of_children() {
    // Arrange
    let item = RowItem {
        size: Size::default(),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        name: String::default(),
        incl_fraction: 0.1f32,
        path: Arc::new(PathBuf::from("/some/path")),
        children: vec![],
        parent: None,
        row_index: 0,
    };

    // Act
    let output = format!("{}", item);

    // Assert
    assert_eq!(
        format!(
            "{} with {} children",
            item.path.display(),
            item.children.len()
        ),
        output
    );
}
