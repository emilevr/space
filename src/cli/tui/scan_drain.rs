use super::scan_worker::{self, ScanMessage};
use crate::cli::row_item::RowItem;
use crate::cli::view_state::{
    scan_helpers::{build_child_prefix_base, find_child_by_name, resort_child_in_parent},
    ViewState,
};
use crossfire::TryRecvError;
use std::{cell::RefCell, rc::Rc, time::Instant};

/// Drains pending scan messages from the receiver into the view state.
/// Returns `true` when all scans are complete (`active_scan_count` reaches 0).
pub(super) fn drain_scan_channel(
    receiver: &scan_worker::ScanReceiver,
    view_state: &mut ViewState,
    deadline: Instant,
    active_scan_count: &mut usize,
) -> bool {
    let saved = view_state.save_selected_path();
    let mut items_added = false;
    let mut all_complete = false;
    let mut needs_resort: Vec<(Rc<RefCell<RowItem>>, String)> = Vec::new();
    let mut batch_parents: Vec<Rc<RefCell<RowItem>>> = Vec::new();
    let mut messages_processed: usize = 0;

    loop {
        // Check the time budget every 50 messages to avoid calling Instant::now()
        // on every iteration while still yielding promptly.
        messages_processed += 1;
        if messages_processed % 50 == 0 && Instant::now() >= deadline {
            break;
        }
        match receiver.try_recv() {
            Ok(ScanMessage::Item(dir_item)) => {
                view_state.add_scanned_item(dir_item);
                items_added = true;
            }
            Ok(ScanMessage::ChildItem(child_item)) => {
                view_state.add_scanned_child(child_item);
                items_added = true;
            }
            Ok(ScanMessage::DescendantBatch {
                ancestor_path,
                children,
            }) => {
                handle_descendant_batch(
                    view_state,
                    &ancestor_path,
                    children,
                    &mut needs_resort,
                    &mut batch_parents,
                );
                items_added = true;
            }
            Ok(ScanMessage::AccessDenied(ancestor_path)) => {
                view_state.mark_access_denied(&ancestor_path);
            }
            Ok(ScanMessage::ChildScanComplete(name)) => {
                view_state.mark_child_scan_complete(&name);
            }
            Ok(ScanMessage::Complete) => {
                *active_scan_count = active_scan_count.saturating_sub(1);
                if *active_scan_count == 0 {
                    finalize_scan(view_state);
                    all_complete = true;
                    break;
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                *active_scan_count = 0;
                finalize_scan(view_state);
                all_complete = true;
                break;
            }
        }
    }

    if !all_complete && items_added {
        apply_post_drain_updates(view_state, &needs_resort, &batch_parents);
    }

    if let (Some((path, screen_position)), true) = (saved, items_added) {
        view_state.restore_selection(&path, screen_position);
    }

    all_complete
}

fn handle_descendant_batch(
    view_state: &mut ViewState,
    ancestor_path: &[String],
    children: Vec<space_rs::DirectoryItem>,
    needs_resort: &mut Vec<(Rc<RefCell<RowItem>>, String)>,
    batch_parents: &mut Vec<Rc<RefCell<RowItem>>>,
) {
    if let Some(root) = view_state.item_tree.last().cloned() {
        let mut current = root.clone();
        for name in ancestor_path {
            record_resort(needs_resort, &current, name);
            match find_child_by_name(&current, name) {
                Some(next) => current = next,
                None => break,
            }
        }
        record_batch_parent(batch_parents, &current);
    }
    view_state.add_scanned_descendant_batch(ancestor_path, children);
}

fn finalize_scan(view_state: &mut ViewState) {
    view_state.is_scanning = false;
    view_state.sort_root_children();
    view_state.recalculate_fractions();
    if view_state.filter_regex.is_some() {
        view_state.apply_regex_filter();
    }
}

fn apply_post_drain_updates(
    view_state: &mut ViewState,
    needs_resort: &[(Rc<RefCell<RowItem>>, String)],
    batch_parents: &[Rc<RefCell<RowItem>>],
) {
    for (parent, child_name) in needs_resort {
        let parent_prefix = parent.borrow().tree_prefix.clone();
        let prefix_for_children = build_child_prefix_base(&parent_prefix);
        resort_child_in_parent(&mut parent.borrow_mut(), child_name, &prefix_for_children);
    }
    for parent in batch_parents {
        sort_children_single_level(parent);
    }
    view_state.recalculate_fractions();
    if view_state.filter_regex.is_some() {
        view_state.apply_regex_filter();
    }
}

/// Records a `(parent, child_name)` pair for post-drain resorting, deduplicating
/// by parent pointer identity and child name.
fn record_resort(
    needs_resort: &mut Vec<(Rc<RefCell<RowItem>>, String)>,
    parent: &Rc<RefCell<RowItem>>,
    child_name: &str,
) {
    let parent_ptr = Rc::as_ptr(parent);
    let already = needs_resort
        .iter()
        .any(|(p, n)| Rc::as_ptr(p) == parent_ptr && n == child_name);
    if !already {
        needs_resort.push((parent.clone(), child_name.to_string()));
    }
}

/// Records a parent node that received a descendant batch, deduplicating by
/// pointer identity.
fn record_batch_parent(
    batch_parents: &mut Vec<Rc<RefCell<RowItem>>>,
    parent: &Rc<RefCell<RowItem>>,
) {
    let ptr = Rc::as_ptr(parent);
    if !batch_parents.iter().any(|p| Rc::as_ptr(p) == ptr) {
        batch_parents.push(parent.clone());
    }
}

/// Sorts a single parent's children by descending size, then ascending name,
/// and rebuilds their tree prefixes.
fn sort_children_single_level(parent: &Rc<RefCell<RowItem>>) {
    let mut parent_ref = parent.borrow_mut();
    parent_ref.children.sort_by(|a, b| {
        let a_ref = a.borrow();
        let b_ref = b.borrow();
        match b_ref.size.get_value().cmp(&a_ref.size.get_value()) {
            std::cmp::Ordering::Equal => a_ref.path_segment.cmp(&b_ref.path_segment),
            ord => ord,
        }
    });
    let parent_prefix = parent_ref.tree_prefix.clone();
    let prefix_for_children = build_child_prefix_base(&parent_prefix);
    let child_count = parent_ref.children.len();
    for (i, child) in parent_ref.children.iter().enumerate() {
        let is_last = i == child_count - 1;
        let mut child_prefix = prefix_for_children.clone();
        if !is_last {
            child_prefix.push('\u{251C}');
        }
        child
            .borrow_mut()
            .update_tree_prefix(&child_prefix, is_last);
    }
}
