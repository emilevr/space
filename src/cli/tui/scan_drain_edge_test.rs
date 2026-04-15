use super::scan_drain::drain_scan_channel;
use crate::cli::scan_worker::ScanMessage;
use crate::cli::scan_worker::ScanReceiver;
use crate::cli::view_state::ViewState;
use crossfire::mpsc as cf_mpsc;
use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::time::{Duration, Instant};

fn test_deadline() -> Instant {
    Instant::now() + Duration::from_secs(1)
}

fn test_drain(receiver: &ScanReceiver, view_state: &mut ViewState) -> bool {
    let mut count = 1usize;
    drain_scan_channel(receiver, view_state, test_deadline(), &mut count)
}

fn make_file_dir_item(path_segment: &str, size: u64) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size),
        descendant_count: 0,
        children: vec![],
    }
}

// ─── Edge-case tests for drain_scan_channel ──────────────────────────────────

#[test]
fn drain_scan_channel_ignores_items_queued_after_complete() {
    // Verifies that the `break` on Complete stops processing of any further
    // messages already in the channel buffer for the same tick.  This is the
    // unit-level guarantee that underpins the render_loop fix: once
    // drain_scan_channel returns true, the receiver is dropped (set to None),
    // so those messages are never processed in any subsequent tick either.
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();
    sender
        .send(ScanMessage::Item(make_file_dir_item(
            "before_complete",
            1000,
        )))
        .unwrap();
    sender.send(ScanMessage::Complete).unwrap();
    // This message arrives in the buffer AFTER Complete and must be ignored.
    sender
        .send(ScanMessage::Item(make_file_dir_item(
            "after_complete",
            2000,
        )))
        .unwrap();

    let complete = test_drain(&receiver, &mut view_state);

    assert!(complete, "Should signal completion");
    // Only the item sent before Complete should be in the tree.
    assert_eq!(
        1,
        view_state.item_tree.len(),
        "Item sent after Complete must not be processed"
    );
    assert_eq!(
        "before_complete",
        view_state.item_tree[0].borrow().path_segment
    );
}

// ─── Selection-stability tests for drain_scan_channel ────────────────────────

fn make_dir_item_for_scan(path_segment: &str, size: u64) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: space_rs::DirectoryItemType::Directory,
        size_in_bytes: Size::new(size),
        descendant_count: 0,
        children: vec![],
    }
}

/// Builds a ViewState with a scan-root directory and zero or more file children
/// already added, leaving the view updated.
fn make_scanning_view_state(children: &[(&str, u64)]) -> ViewState {
    let mut view_state = ViewState {
        is_scanning: true,
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_dir_item_for_scan("/root", 0));
    for (name, size) in children {
        view_state.add_scanned_child(make_file_dir_item(name, *size));
    }
    view_state.update_visible_rows();
    view_state
}

#[test]
fn drain_scan_channel_preserves_selection_when_child_inserted_after_selected_item() {
    // Tree before: root(0), big_child(1).  Select big_child.
    let mut view_state = make_scanning_view_state(&[("big_child", 100)]);
    view_state.table_selected_index = 1;
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    // Insert small_child(10) -> goes AFTER big_child in sorted order.
    sender
        .send(ScanMessage::ChildItem(make_file_dir_item(
            "small_child",
            10,
        )))
        .unwrap();

    test_drain(&receiver, &mut view_state);

    // big_child is still selected.
    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("big_child", selected.borrow().path_segment);
}

#[test]
fn drain_scan_channel_preserves_selection_when_child_inserted_before_selected_item() {
    // Tree before: root(0), small_child(1).  Select small_child.
    let mut view_state = make_scanning_view_state(&[("small_child", 10)]);
    view_state.table_selected_index = 1;
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    // Insert big_child(100) -> goes BEFORE small_child in sorted order.
    // Without selection tracking, table_selected_index=1 would land on big_child.
    sender
        .send(ScanMessage::ChildItem(make_file_dir_item("big_child", 100)))
        .unwrap();

    test_drain(&receiver, &mut view_state);

    // small_child is still selected despite the insertion above it.
    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("small_child", selected.borrow().path_segment);
}

#[test]
fn drain_scan_channel_preserves_root_selection_when_children_added() {
    // Root selected at index 0.
    let mut view_state = make_scanning_view_state(&[]);
    view_state.table_selected_index = 0;
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    sender
        .send(ScanMessage::ChildItem(make_file_dir_item("child_a", 200)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_file_dir_item("child_b", 50)))
        .unwrap();

    test_drain(&receiver, &mut view_state);

    // Root remains selected.
    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("/root", selected.borrow().path_segment);
    assert_eq!(0, view_state.table_selected_index);
}

#[test]
fn drain_scan_channel_child_scan_complete_clears_is_scanning() {
    let mut view_state = ViewState {
        is_scanning: true,
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    // Add root and a directory child - is_scanning=true is set by add_scanned_child.
    sender
        .send(ScanMessage::Item(make_dir_item_for_scan("/root", 0)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_dir_item_for_scan(
            "dir_child",
            0,
        )))
        .unwrap();
    test_drain(&receiver, &mut view_state);

    {
        let root = view_state.item_tree[0].borrow();
        assert!(
            root.children[0].borrow().is_scanning,
            "dir_child should be is_scanning=true after add_scanned_child"
        );
    }

    // ChildScanComplete should clear the flag.
    let (sender2, receiver2) = cf_mpsc::unbounded_blocking();
    sender2
        .send(ScanMessage::ChildScanComplete("dir_child".to_string()))
        .unwrap();
    test_drain(&receiver2, &mut view_state);

    let root = view_state.item_tree[0].borrow();
    assert!(
        !root.children[0].borrow().is_scanning,
        "dir_child should be is_scanning=false after ChildScanComplete"
    );
}

#[test]
fn drain_scan_channel_processes_grandchild_item_and_adds_to_parent() {
    let mut view_state = ViewState {
        is_scanning: true,
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    // Scanning protocol: root Item -> empty ChildItem shell -> GrandchildItem.
    sender
        .send(ScanMessage::Item(make_dir_item_for_scan("/root", 0)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_dir_item_for_scan(
            "parent_dir",
            0,
        )))
        .unwrap();
    sender
        .send(ScanMessage::DescendantBatch {
            ancestor_path: vec!["parent_dir".to_string()],
            children: vec![make_file_dir_item("grandchild.txt", 500)],
        })
        .unwrap();

    test_drain(&receiver, &mut view_state);

    let root = view_state.item_tree[0].borrow();
    assert_eq!(500, root.size.get_value());
    let parent = root
        .children
        .iter()
        .find(|c| c.borrow().path_segment == "parent_dir")
        .expect("parent_dir not found")
        .borrow();
    assert_eq!(1, parent.children.len());
    assert_eq!("grandchild.txt", parent.children[0].borrow().path_segment);
    assert_eq!(500, parent.size.get_value());
}

#[test]
fn drain_scan_channel_descendant_batch_sets_items_added_and_triggers_selection_restore() {
    // Verifies that a DescendantBatch sets items_added -> restore_selection is called.
    let mut view_state = ViewState {
        is_scanning: true,
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    sender
        .send(ScanMessage::Item(make_dir_item_for_scan("/root", 0)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_dir_item_for_scan("dir_a", 0)))
        .unwrap();

    test_drain(&receiver, &mut view_state);
    // Simulate what the render loop does: rebuild visible rows so the view state
    // is consistent before the next drain (displayable_item_count, visible_row_items).
    view_state.update_visible_rows();

    // Select the root item at screen position 0.
    view_state.table_selected_index = 0;

    let (sender2, receiver2) = cf_mpsc::unbounded_blocking();
    sender2
        .send(ScanMessage::DescendantBatch {
            ancestor_path: vec!["dir_a".to_string()],
            children: vec![make_file_dir_item("file.txt", 300)],
        })
        .unwrap();

    test_drain(&receiver2, &mut view_state);

    // Root was selected at screen position 0; after restore_selection it should remain.
    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("/root", selected.borrow().path_segment);
}

#[test]
fn drain_scan_channel_resorts_children_after_descendant_batch_changes_sizes() {
    // Two root children: dir_a(0) and dir_b(100). dir_b is larger so sorted first.
    // A descendant batch adds 500 bytes under dir_a, making it larger than dir_b.
    // After draining, dir_a should be resorted to appear before dir_b.
    let mut view_state = ViewState {
        is_scanning: true,
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    sender
        .send(ScanMessage::Item(make_dir_item_for_scan("/root", 0)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_dir_item_for_scan("dir_a", 0)))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_file_dir_item("dir_b", 100)))
        .unwrap();
    test_drain(&receiver, &mut view_state);

    // Before: dir_b (100) is first, dir_a (0) is second.
    {
        let root = view_state.item_tree[0].borrow();
        assert_eq!("dir_b", root.children[0].borrow().path_segment);
        assert_eq!("dir_a", root.children[1].borrow().path_segment);
    }

    let (sender2, receiver2) = cf_mpsc::unbounded_blocking();
    sender2
        .send(ScanMessage::DescendantBatch {
            ancestor_path: vec!["dir_a".to_string()],
            children: vec![make_file_dir_item("big_file.txt", 500)],
        })
        .unwrap();
    test_drain(&receiver2, &mut view_state);

    // After: dir_a (500) should now be first, dir_b (100) second.
    let root = view_state.item_tree[0].borrow();
    assert_eq!(
        "dir_a",
        root.children[0].borrow().path_segment,
        "dir_a should be resorted to first after growing larger than dir_b"
    );
    assert_eq!(
        "dir_b",
        root.children[1].borrow().path_segment,
        "dir_b should now be second"
    );
}

#[test]
fn drain_scan_channel_does_not_restore_selection_on_empty_channel() {
    // No messages in channel; items_added stays false -> restore_selection not called.
    let mut view_state = make_scanning_view_state(&[("existing_child", 100)]);
    view_state.table_selected_index = 1;
    let (_sender, receiver) = cf_mpsc::unbounded_blocking::<ScanMessage>();

    test_drain(&receiver, &mut view_state);

    // table_selected_index unchanged.
    assert_eq!(1, view_state.table_selected_index);
    assert!(view_state.is_scanning);
}
