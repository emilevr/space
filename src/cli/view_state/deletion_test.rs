use super::{remove_dir_all_cancellable, subtract_item_tree_size};
use crate::cli::view_state::{DeletionState, ViewState};
use crate::cli::view_state_test_utils::{
    get_row_index_by_name, make_test_view_state, make_test_view_state_with_height,
    select_item_by_name,
};
use crate::test_directory_utils::delete_test_directory_tree;
use rstest::rstest;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[rstest]
fn subtract_item_tree_size_subtracts_value_from_self_and_ancestors() -> anyhow::Result<()> {
    // Arrange
    let value_to_subtract = 10000u64;
    let (view_state, temp_dir_path) = make_test_view_state_with_height(10, 0, 0f32)?;

    // Act
    subtract_item_tree_size(&view_state.visible_row_items[4], value_to_subtract);

    // Assert
    assert_eq!(
        13000,
        view_state.visible_row_items[4].borrow().size.get_value()
    );
    assert_eq!(
        170000,
        view_state.visible_row_items[1].borrow().size.get_value()
    );
    assert_eq!(
        170000,
        view_state.visible_row_items[0].borrow().size.get_value()
    );
    assert_eq!(
        24000,
        view_state.visible_row_items[3].borrow().size.get_value()
    );
    assert_eq!(
        25000,
        view_state.visible_row_items[2].borrow().size.get_value()
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case("1.3", 25, 157000)] // Delete a directory with 3 sub-items
#[case("1.4", 28, 158000)] // Delete a single file
#[case("1.5", 18, 162000)] // Delete a directory with 10 sub-items
#[case("1.5.3.5", 28, 180000)] // Delete an empty directory
#[case("1.11", 28, 180000)] // Deletes symbolic link to dir, but not the tree it points to
fn delete_selected_item_deletes_only_the_selected_item_or_tree_and_updates_sizes(
    #[case] selected_item_name: &str,
    #[case] expected_final_item_count: usize,
    #[case] expected_total_size: u64,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.delete_selected_item();
    view_state.update_visible_rows();

    // Assert
    assert_eq!(None, get_row_index_by_name(selected_item_name, &view_state));
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);
    assert_eq!(
        expected_total_size,
        view_state.item_tree[0].borrow().size.get_value()
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

// ─── Async deletion flow tests ────────────────────────────────────────────────

#[test]
fn start_async_deletion_and_check_complete_deletes_file() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // "1.4" is a single file in the test tree.
    select_item_by_name("1.4", &mut view_state)?;
    let selected_path = view_state.get_selected_item().unwrap().borrow().get_path();
    assert!(selected_path.exists(), "File should exist before deletion");

    view_state.start_async_deletion();
    // In test mode, deletion runs synchronously, so the result is immediately available.
    view_state.check_deletion_complete();

    assert!(
        !selected_path.exists(),
        "File should be gone after async deletion completes"
    );
    assert!(
        get_row_index_by_name("1.4", &view_state).is_none(),
        "Item should be removed from view state"
    );
    assert_eq!(
        DeletionState::Idle,
        view_state.deletion_state,
        "Deletion state should reset to Idle"
    );
    assert!(!view_state.show_delete_dialog, "Dialog should be closed");

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[test]
fn start_async_deletion_and_check_complete_deletes_directory() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // "1.3" is a directory with sub-items.
    select_item_by_name("1.3", &mut view_state)?;
    let selected_path = view_state.get_selected_item().unwrap().borrow().get_path();
    assert!(
        selected_path.exists(),
        "Directory should exist before deletion"
    );

    view_state.start_async_deletion();
    view_state.check_deletion_complete();

    assert!(
        !selected_path.exists(),
        "Directory should be gone after async deletion completes"
    );
    assert!(
        get_row_index_by_name("1.3", &view_state).is_none(),
        "Item should be removed from view state"
    );
    assert_eq!(DeletionState::Idle, view_state.deletion_state);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[test]
fn cancel_deletion_sets_cancelling_state_and_sets_flag() {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let mut view_state = ViewState {
        deletion_state: DeletionState::InProgress,
        deletion_cancel_flag: Some(cancel_flag.clone()),
        ..Default::default()
    };

    view_state.cancel_deletion();

    assert_eq!(
        DeletionState::Cancelling,
        view_state.deletion_state,
        "State should change to Cancelling"
    );
    assert!(
        cancel_flag.load(Ordering::Relaxed),
        "Cancel flag must be set to true"
    );
}

#[test]
fn check_deletion_complete_with_no_receiver_is_a_no_op() {
    let mut view_state = ViewState::default();
    // No receiver set - should not panic or change state.
    view_state.check_deletion_complete();
    assert_eq!(DeletionState::Idle, view_state.deletion_state);
}

// ─── remove_dir_all_cancellable unit tests ────────────────────────────────────

#[test]
fn remove_dir_all_cancellable_completes_successfully_without_cancel() -> anyhow::Result<()> {
    let temp_dir = std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    std::fs::write(temp_dir.join("a.txt"), "hello")?;
    std::fs::write(temp_dir.join("b.txt"), "world")?;
    let sub = temp_dir.join("sub");
    std::fs::create_dir_all(&sub)?;
    std::fs::write(sub.join("c.txt"), "nested")?;

    let cancel_flag = AtomicBool::new(false);
    let cancelled = remove_dir_all_cancellable(&temp_dir, &cancel_flag)?;

    assert!(!cancelled, "Should complete without cancellation");
    assert!(!temp_dir.exists(), "Directory should be fully removed");
    Ok(())
}

#[test]
fn remove_dir_all_cancellable_returns_cancelled_when_flag_preset() -> anyhow::Result<()> {
    let temp_dir = std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    std::fs::write(temp_dir.join("file.txt"), "data")?;

    let cancel_flag = AtomicBool::new(true); // pre-cancelled
    let cancelled = remove_dir_all_cancellable(&temp_dir, &cancel_flag)?;

    assert!(cancelled, "Should report cancellation immediately");
    // Directory may still exist (partial deletion is acceptable after cancel).
    // Clean up whatever remains.
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn remove_dir_all_cancellable_removes_symlink_without_following_it() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    // Create a target file outside the directory being deleted.
    let target =
        std::env::temp_dir().join(format!("space_symlink_target_{}", uuid::Uuid::new_v4()));
    std::fs::write(&target, "target content")?;

    // Create the directory to delete, containing a symlink to the target.
    let temp_dir = std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let link = temp_dir.join("link_to_target");
    symlink(&target, &link)?;

    let cancel_flag = AtomicBool::new(false);
    let cancelled = remove_dir_all_cancellable(&temp_dir, &cancel_flag)?;

    assert!(!cancelled, "Should complete without cancellation");
    assert!(!link.exists(), "Symlink should be removed");
    assert!(!temp_dir.exists(), "Containing directory should be removed");
    assert!(
        target.exists(),
        "Symlink target must NOT be deleted - symlinks must not be followed"
    );

    // Clean up target.
    std::fs::remove_file(&target).ok();
    Ok(())
}
