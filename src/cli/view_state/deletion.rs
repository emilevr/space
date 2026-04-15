#[cfg(test)]
#[path = "deletion_test.rs"]
mod deletion_test;

use super::{DeletionResult, DeletionState, ViewState};
use crate::cli::row_item::RowItem;
use log::error;
#[cfg(not(test))]
use std::thread;
use std::{
    cell::RefCell,
    fs, io,
    path::Path,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

impl ViewState {
    pub(crate) fn start_async_deletion(&mut self) {
        let Some(selected_item) = self.get_selected_item() else {
            return;
        };
        let path = selected_item.borrow().get_path();
        let is_dir = path.is_dir();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = crossfire::mpsc::unbounded_blocking();

        let flag_clone = cancel_flag.clone();
        let do_delete = move || {
            let result = if is_dir {
                match remove_dir_all_cancellable(&path, &flag_clone) {
                    Ok(false) => DeletionResult::Success,
                    Ok(true) => DeletionResult::Cancelled,
                    Err(e) => DeletionResult::Error(e.to_string()),
                }
            } else {
                match fs::remove_file(&path) {
                    Ok(()) => DeletionResult::Success,
                    Err(e) => DeletionResult::Error(e.to_string()),
                }
            };
            let _ = sender.send(result);
        };

        // In tests, run deletion synchronously so the result is immediately
        // available to check_deletion_complete on the next tick.
        #[cfg(test)]
        do_delete();
        #[cfg(not(test))]
        thread::spawn(do_delete);

        self.deletion_state = DeletionState::InProgress;
        self.deletion_cancel_flag = Some(cancel_flag);
        self.deletion_receiver = Some(receiver);
    }

    pub(crate) fn cancel_deletion(&mut self) {
        if let Some(ref flag) = self.deletion_cancel_flag {
            flag.store(true, Ordering::Relaxed);
        }
        self.deletion_state = DeletionState::Cancelling;
    }

    pub(crate) fn check_deletion_complete(&mut self) {
        let result = if let Some(ref receiver) = self.deletion_receiver {
            receiver.try_recv().ok()
        } else {
            None
        };

        let Some(result) = result else {
            return;
        };

        match result {
            DeletionResult::Success => {
                if let Some(selected_item) = self.get_selected_item() {
                    let parent = selected_item.borrow().parent.clone();
                    if let Some(parent) = parent {
                        self.remove_child_item(&parent, &selected_item);
                    } else {
                        self.remove_top_level_item(&selected_item);
                    }
                }
            }
            DeletionResult::Cancelled => {
                self.status_message = Some("Deletion cancelled".to_string());
                // Rescan to pick up any partially deleted state.
                self.prepare_rescan();
            }
            DeletionResult::Error(ref e) => {
                error!("Deletion of item failed: {}", e);
                self.status_message = Some(format!("Deletion failed: {e}"));
                // Rescan to reflect actual disk state after partial failure.
                self.prepare_rescan();
            }
        }

        self.reset_deletion_state();
    }

    fn reset_deletion_state(&mut self) {
        self.deletion_state = DeletionState::Idle;
        self.deletion_cancel_flag = None;
        self.deletion_receiver = None;
        self.show_delete_dialog = false;
    }

    #[cfg(test)]
    pub(crate) fn delete_selected_item(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let selected_item_ref = selected_item.borrow();
            let path = selected_item_ref.get_path();

            let remove_result = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            match remove_result {
                Ok(()) => {
                    if let Some(parent) = &selected_item_ref.parent {
                        self.remove_child_item(parent, &selected_item);
                    } else {
                        self.remove_top_level_item(&selected_item);
                    }
                }
                Err(e) => {
                    // TODO: Push the error into some sort of error stream and expose in UI.
                    error!("Deletion of item failed: {}", e);
                }
            }
        }
    }

    fn remove_child_item(
        &mut self,
        parent: &std::rc::Weak<RefCell<RowItem>>,
        selected_item: &Rc<RefCell<RowItem>>,
    ) {
        if let Some(parent) = parent.upgrade() {
            let mut child_index = None;
            {
                let children = &parent.borrow().children;
                for (i, child) in children.iter().enumerate() {
                    if Rc::ptr_eq(child, selected_item) {
                        child_index = Some(i);
                        break;
                    }
                }
            }

            if let Some(child_index) = child_index {
                {
                    let mut parent_ref = parent.borrow_mut();
                    let children = &mut parent_ref.children;
                    children.remove(child_index);
                }

                // Update the total number of items in the tree
                self.update_total_items_in_tree();
                self.visible_rows_dirty = true;

                // Lastly remove the size of the deleted item from self and ancestors.
                subtract_item_tree_size(&parent, selected_item.borrow().size.get_value());
            }
        }
    }

    fn remove_top_level_item(&mut self, selected_item: &Rc<RefCell<RowItem>>) {
        let mut item_index = None;
        for (i, item) in self.item_tree.iter().enumerate() {
            if Rc::ptr_eq(item, selected_item) {
                item_index = Some(i);
                break;
            }
        }
        if let Some(item_index) = item_index {
            self.item_tree.remove(item_index);
            self.update_total_items_in_tree();
            self.visible_rows_dirty = true;
        }
    }
}

/// Recursively removes a directory, checking the cancel flag between entries.
/// Symlinks are removed directly without following them into their targets.
/// Returns `Ok(true)` if cancelled, `Ok(false)` if completed successfully.
fn remove_dir_all_cancellable(path: &Path, cancel_flag: &AtomicBool) -> io::Result<bool> {
    for entry in fs::read_dir(path)? {
        if cancel_flag.load(Ordering::Relaxed) {
            return Ok(true);
        }

        let entry = entry?;
        let file_type = entry.metadata()?.file_type();

        if file_type.is_symlink() {
            // Remove the symlink itself, not its target contents.
            // On Windows, directory symlinks must be removed with remove_dir
            // while file symlinks use remove_file.  Since the target may
            // already be deleted (broken symlink), we try remove_dir first
            // and fall back to remove_file.
            let entry_path = entry.path();
            if fs::remove_dir(&entry_path).is_err() {
                fs::remove_file(&entry_path)?;
            }
        } else if file_type.is_dir() {
            if remove_dir_all_cancellable(&entry.path(), cancel_flag)? {
                return Ok(true);
            }
        } else {
            fs::remove_file(entry.path())?;
        }
    }

    if cancel_flag.load(Ordering::Relaxed) {
        return Ok(true);
    }

    fs::remove_dir(path)?;
    Ok(false)
}

pub(crate) fn subtract_item_tree_size(item: &RefCell<RowItem>, size: u64) {
    let mut item_ref = item.borrow_mut();
    item_ref.size.subtract(size);

    if let Some(parent) = &item_ref.parent {
        if let Some(parent) = parent.upgrade() {
            let parent = parent.as_ref();
            subtract_item_tree_size(parent, size);
        }
    }
}
