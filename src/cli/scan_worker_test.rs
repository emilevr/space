use super::ScanReceiver;
use super::{spawn_scan, ScanMessage};
use crate::test_directory_utils::{create_test_directory_tree, delete_test_directory_tree};
use space_rs::DirectoryItemType;
use std::{
    fs,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

fn test_spawn_scan(paths: Vec<std::path::PathBuf>, should_exit: Arc<AtomicBool>) -> ScanReceiver {
    let (sender, receiver) = crossfire::mpsc::unbounded_blocking();
    spawn_scan(paths, should_exit, sender);
    receiver
}

#[test]
fn spawn_scan_with_empty_paths_sends_only_complete() {
    let should_exit = Arc::new(AtomicBool::new(false));
    let receiver = test_spawn_scan(vec![], should_exit);

    let msg = receiver
        .recv_timeout(RECV_TIMEOUT)
        .expect("Expected Complete message");
    assert!(
        matches!(msg, ScanMessage::Complete),
        "Expected Complete, got an Item"
    );
    // Channel should be closed - no further messages.
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn spawn_scan_with_valid_directory_sends_item_then_children_then_complete() -> anyhow::Result<()> {
    let temp_dir = create_test_directory_tree()?;
    let should_exit = Arc::new(AtomicBool::new(false));

    let receiver = test_spawn_scan(vec![temp_dir.clone()], should_exit);

    // First message: root Item (empty shell).
    let msg = receiver
        .recv_timeout(RECV_TIMEOUT)
        .expect("Expected Item message");
    assert!(
        matches!(msg, ScanMessage::Item(_)),
        "Expected Item, got something else"
    );

    // Then one or more ChildItem messages, followed by Complete.
    let mut child_count = 0;
    loop {
        let msg = receiver
            .recv_timeout(RECV_TIMEOUT)
            .expect("Expected message");
        match msg {
            ScanMessage::ChildItem(_) => child_count += 1,
            ScanMessage::DescendantBatch { .. }
            | ScanMessage::AccessDenied(_)
            | ScanMessage::ChildScanComplete(_) => {}
            ScanMessage::Complete => break,
            ScanMessage::Item(_) => panic!("Unexpected second Item message"),
        }
    }

    assert!(
        child_count > 0,
        "Expected at least one ChildItem for a non-empty directory"
    );

    delete_test_directory_tree(&temp_dir);
    Ok(())
}

#[test]
fn spawn_scan_with_should_exit_set_sends_only_complete() -> anyhow::Result<()> {
    let temp_dir = create_test_directory_tree()?;
    // should_exit already true - loop should break before scanning any path.
    let should_exit = Arc::new(AtomicBool::new(true));

    let receiver = test_spawn_scan(vec![temp_dir.clone()], should_exit);

    let msg = receiver
        .recv_timeout(RECV_TIMEOUT)
        .expect("Expected Complete message");
    assert!(
        matches!(msg, ScanMessage::Complete),
        "Expected Complete when should_exit is set"
    );
    assert!(
        receiver.recv_timeout(Duration::from_millis(200)).is_err(),
        "Expected no more messages after Complete"
    );

    delete_test_directory_tree(&temp_dir);
    Ok(())
}

#[test]
fn spawn_scan_deduplicates_duplicate_paths() -> anyhow::Result<()> {
    let temp_dir = create_test_directory_tree()?;
    let should_exit = Arc::new(AtomicBool::new(false));

    // Pass the same path twice - should be deduplicated to one scan.
    let receiver = test_spawn_scan(vec![temp_dir.clone(), temp_dir.clone()], should_exit);

    // Exactly one Item (root shell).
    let msg = receiver.recv_timeout(RECV_TIMEOUT).expect("Expected Item");
    assert!(matches!(msg, ScanMessage::Item(_)));

    // Drain ChildItems, then Complete.
    loop {
        let msg = receiver
            .recv_timeout(RECV_TIMEOUT)
            .expect("Expected message");
        match msg {
            ScanMessage::ChildItem(_)
            | ScanMessage::DescendantBatch { .. }
            | ScanMessage::AccessDenied(_)
            | ScanMessage::ChildScanComplete(_) => continue,
            ScanMessage::Complete => break,
            ScanMessage::Item(_) => panic!("Unexpected second Item - path should be deduplicated"),
        }
    }

    // Nothing else.
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());

    delete_test_directory_tree(&temp_dir);
    Ok(())
}

#[test]
fn spawn_scan_with_multiple_paths_sends_item_per_path_then_complete() -> anyhow::Result<()> {
    let temp_dir1 = create_test_directory_tree()?;
    let temp_dir2 = create_test_directory_tree()?;
    let should_exit = Arc::new(AtomicBool::new(false));

    let receiver = test_spawn_scan(vec![temp_dir1.clone(), temp_dir2.clone()], should_exit);

    let mut item_count = 0;
    loop {
        let msg = receiver
            .recv_timeout(RECV_TIMEOUT)
            .expect("Expected message");
        match msg {
            ScanMessage::Item(_) => item_count += 1,
            ScanMessage::ChildItem(_)
            | ScanMessage::DescendantBatch { .. }
            | ScanMessage::AccessDenied(_)
            | ScanMessage::ChildScanComplete(_) => continue,
            ScanMessage::Complete => break,
        }
    }

    assert_eq!(
        2, item_count,
        "Expected exactly 2 root items for 2 distinct paths"
    );

    delete_test_directory_tree(&temp_dir1);
    delete_test_directory_tree(&temp_dir2);
    Ok(())
}

#[test]
fn spawn_scan_with_empty_directory_sends_item_then_complete_no_children() -> anyhow::Result<()> {
    let temp_dir = std::env::temp_dir().join(format!("space_{}", uuid::Uuid::new_v4()));
    fs::create_dir(&temp_dir)?;
    let should_exit = Arc::new(AtomicBool::new(false));

    let receiver = test_spawn_scan(vec![temp_dir.clone()], should_exit);

    // First message: root Item (empty shell for the empty directory).
    let msg = receiver
        .recv_timeout(RECV_TIMEOUT)
        .expect("Expected Item message");
    assert!(
        matches!(msg, ScanMessage::Item(_)),
        "Expected Item for empty directory root"
    );

    // Next message must be Complete - no ChildItems for an empty directory.
    let msg = receiver
        .recv_timeout(RECV_TIMEOUT)
        .expect("Expected Complete message");
    assert!(
        matches!(msg, ScanMessage::Complete),
        "Expected Complete with no ChildItems for an empty directory"
    );

    // Channel should now be closed.
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn spawn_scan_with_symlink_child_sends_child_item_with_symbolic_link_type_not_expanded(
) -> anyhow::Result<()> {
    // "1/1.11" is a symlink in the test tree. Scanning "1/" should emit a ChildItem
    // for the symlink with SymbolicLink type - it must NOT be expanded into grandchildren.
    let temp_dir = create_test_directory_tree()?;
    let scan_dir = temp_dir.join("1");
    let should_exit = Arc::new(AtomicBool::new(false));

    let receiver = test_spawn_scan(vec![scan_dir.clone()], should_exit);

    // Drain all messages. Track whether "1.11" (a direct symlink child of the scan root)
    // is sent as a ChildItem, and whether it is incorrectly expanded further.
    let mut saw_1_11_as_child_item = false;
    let mut saw_1_11_grandchild_messages = false;
    let mut saw_1_11_child_scan_complete = false;
    loop {
        let msg = receiver
            .recv_timeout(RECV_TIMEOUT)
            .expect("Expected message");
        match msg {
            ScanMessage::ChildItem(item) => {
                if item.path_segment == "1.11" {
                    assert_eq!(
                        DirectoryItemType::SymbolicLink,
                        item.item_type,
                        "'1.11' ChildItem should have SymbolicLink type"
                    );
                    saw_1_11_as_child_item = true;
                }
            }
            ScanMessage::DescendantBatch {
                ref ancestor_path,
                ref children,
            } => {
                if ancestor_path.contains(&"1.11".to_string())
                    || children.iter().any(|c| c.path_segment == "1.11")
                {
                    saw_1_11_grandchild_messages = true;
                }
            }
            ScanMessage::ChildScanComplete(ref name) => {
                if name == "1.11" {
                    saw_1_11_child_scan_complete = true;
                }
            }
            ScanMessage::Item(_) | ScanMessage::AccessDenied(_) => {}
            ScanMessage::Complete => break,
        }
    }

    assert!(
        saw_1_11_as_child_item,
        "symlink '1.11' should be sent as a ChildItem with SymbolicLink type"
    );
    assert!(
        !saw_1_11_grandchild_messages,
        "symlink '1.11' should not be expanded into grandchild messages"
    );
    assert!(
        !saw_1_11_child_scan_complete,
        "symlink '1.11' should not generate a ChildScanComplete (symlinks are not scanned)"
    );

    delete_test_directory_tree(&temp_dir);
    Ok(())
}
