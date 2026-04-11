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

fn make_dir_item(path_segment: &str) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::default(),
        descendant_count: 0,
        children: vec![],
    }
}

/// Builds a scanning view state with a root and the given children already added.
fn make_scanning_view(visible_height: usize, children: Vec<DirectoryItem>) -> ViewState {
    let mut vs = ViewState {
        is_scanning: true,
        visible_height,
        table_width: 80,
        ..Default::default()
    };
    vs.add_scanned_item(make_dir_item("/root"));
    for child in children {
        vs.add_scanned_child(child);
    }
    vs
}

#[test]
fn auto_expand_does_nothing_when_visible_height_is_zero() {
    let mut vs = make_scanning_view(0, vec![make_file_item("a", 100)]);
    vs.try_auto_expand_to_fill();
    // Root was auto-expanded by insert_child_into_root, nothing else to check
    // except that it didn't panic.
    assert!(!vs.auto_expand_done);
}

#[test]
fn auto_expand_expands_root_children_when_they_fit() {
    // visible_height = 20.  Root + 2 dir children = 3 rows.
    // Each dir child gets 2 grandchildren via descendant batch -> expanding
    // them adds 4 rows -> total 7, fits in 20.
    let mut vs = make_scanning_view(20, vec![make_dir_item("dir_a"), make_dir_item("dir_b")]);

    // Add grandchildren via descendant batch.
    vs.add_scanned_descendant_batch(
        &["dir_a".to_string()],
        vec![make_file_item("a1", 50), make_file_item("a2", 30)],
    );
    vs.add_scanned_descendant_batch(
        &["dir_b".to_string()],
        vec![make_file_item("b1", 40), make_file_item("b2", 20)],
    );

    vs.try_auto_expand_to_fill();

    // Both dir_a and dir_b should now be expanded.
    let root = vs.item_tree[0].borrow();
    assert!(
        root.children[0].borrow().expanded,
        "dir_a should be auto-expanded"
    );
    assert!(
        root.children[1].borrow().expanded,
        "dir_b should be auto-expanded"
    );
}

#[test]
fn auto_expand_stops_when_expanding_would_exceed_visible_height() {
    // visible_height = 5.  Root + 2 dir children = 3 rows.
    // Each dir child has 3 grandchildren -> expanding both adds 6 -> total 9 > 5.
    let mut vs = make_scanning_view(5, vec![make_dir_item("dir_a"), make_dir_item("dir_b")]);

    vs.add_scanned_descendant_batch(
        &["dir_a".to_string()],
        vec![
            make_file_item("a1", 50),
            make_file_item("a2", 30),
            make_file_item("a3", 10),
        ],
    );
    vs.add_scanned_descendant_batch(
        &["dir_b".to_string()],
        vec![
            make_file_item("b1", 40),
            make_file_item("b2", 20),
            make_file_item("b3", 10),
        ],
    );

    vs.try_auto_expand_to_fill();

    // Neither should be expanded (expanding both would exceed visible_height).
    let root = vs.item_tree[0].borrow();
    assert!(
        !root.children[0].borrow().expanded,
        "dir_a should NOT be expanded when it would exceed visible_height"
    );
    assert!(
        !root.children[1].borrow().expanded,
        "dir_b should NOT be expanded when it would exceed visible_height"
    );
    assert!(vs.auto_expand_done);
}

#[test]
fn auto_expand_sets_done_when_already_full() {
    // visible_height = 3.  Root + 3 file children = 4 rows >= 3.
    let mut vs = make_scanning_view(
        3,
        vec![
            make_file_item("a", 100),
            make_file_item("b", 50),
            make_file_item("c", 25),
        ],
    );

    vs.try_auto_expand_to_fill();

    assert!(vs.auto_expand_done);
}

#[test]
fn auto_expand_skips_when_frontier_empty_but_not_full() {
    // visible_height = 20.  Root + 1 dir child (no grandchildren yet) = 2 rows.
    // Frontier is empty because dir_a has no children yet.
    let mut vs = make_scanning_view(20, vec![make_dir_item("dir_a")]);

    vs.try_auto_expand_to_fill();

    // Should NOT set auto_expand_done - children may arrive later.
    assert!(
        !vs.auto_expand_done,
        "auto_expand_done should be false when frontier is empty but space remains"
    );
}

#[test]
fn auto_expand_expands_multiple_levels_when_they_fit() {
    // visible_height = 20.  Root + 1 dir child = 2 rows.
    // dir_a has 1 dir grandchild -> expand dir_a adds 1 -> 3 rows.
    // The grandchild has 1 file -> expand grandchild adds 1 -> 4 rows.  Fits.
    let mut vs = make_scanning_view(20, vec![make_dir_item("dir_a")]);

    // dir_a gets a directory grandchild.
    vs.add_scanned_descendant_batch(&["dir_a".to_string()], vec![make_dir_item("sub_dir")]);
    // sub_dir gets a file.
    vs.add_scanned_descendant_batch(
        &["dir_a".to_string(), "sub_dir".to_string()],
        vec![make_file_item("file.txt", 100)],
    );

    vs.try_auto_expand_to_fill();

    let root = vs.item_tree[0].borrow();
    let dir_a = &root.children[0];
    assert!(dir_a.borrow().expanded, "dir_a should be expanded");
    let sub_dir = &dir_a.borrow().children[0];
    assert!(sub_dir.borrow().expanded, "sub_dir should be expanded");
}

#[test]
fn auto_expand_rebuilds_tree_prefixes() {
    let mut vs = make_scanning_view(20, vec![make_dir_item("dir_a")]);

    vs.add_scanned_descendant_batch(
        &["dir_a".to_string()],
        vec![make_file_item("file.txt", 100)],
    );

    vs.try_auto_expand_to_fill();

    // The grandchild should have a non-empty tree prefix after auto-expand
    // triggers a prefix rebuild.
    let root = vs.item_tree[0].borrow();
    let dir_a = &root.children[0];
    let file = &dir_a.borrow().children[0];
    assert!(
        !file.borrow().tree_prefix.is_empty(),
        "Grandchild tree prefix should be rebuilt after auto-expand"
    );
}
