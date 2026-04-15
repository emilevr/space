use crate::cli::row_item::RowItem;
use space_rs::{DirectoryItem, Size};
use std::{cell::RefCell, rc::Rc};

pub(super) fn build_child_row(
    child_item: &DirectoryItem,
    total_size_in_bytes: u64,
    root: &Rc<RefCell<RowItem>>,
    is_directory: bool,
) -> Rc<RefCell<RowItem>> {
    let child_row = RowItem::from_directory_item(
        child_item,
        total_size_in_bytes,
        Some(Rc::downgrade(root)),
        0,
    );
    if is_directory {
        child_row.borrow_mut().is_scanning = true;
    }
    child_row
}

pub(super) fn insert_child_into_root(
    root: &Rc<RefCell<RowItem>>,
    child_row: Rc<RefCell<RowItem>>,
    child_size: u64,
    child_descendant_count: usize,
) {
    let mut root_ref = root.borrow_mut();
    let root_size = root_ref.size.get_value();
    root_ref.size = Size::new(root_size + child_size);
    root_ref.descendant_count += 1 + child_descendant_count;
    root_ref.has_children = true;
    // Only force expand on the first child; preserve user's collapse state after that.
    if root_ref.children.is_empty() {
        root_ref.expanded = true;
    }

    let child_size_val = child_row.borrow().size.get_value();
    root_ref.max_child_size = root_ref.max_child_size.max(child_size_val);
    let child_name = child_row.borrow().path_segment.clone();
    let insert_pos = find_sorted_insert_position(&root_ref.children, child_size_val, &child_name);
    root_ref.children.insert(insert_pos, child_row);

    // Update root prefix when first child is added ("──" -> "─┬").
    if root_ref.children.len() == 1 {
        root_ref.tree_prefix = "─┬".to_string();
    }

    // Targeted prefix update for the inserted child and affected neighbor.
    root_ref.update_inserted_child_prefix(insert_pos, " ");
}

pub(in crate::cli) fn find_child_by_name(
    parent: &Rc<RefCell<RowItem>>,
    name: &str,
) -> Option<Rc<RefCell<RowItem>>> {
    parent
        .borrow()
        .children
        .iter()
        .find(|c| c.borrow().path_segment == name)
        .cloned()
}

/// Walks the tree from `root` following `names` at each level.
/// Returns `None` if any name in the path is not found.
pub(super) fn find_descendant_by_path(
    root: &Rc<RefCell<RowItem>>,
    names: &[String],
) -> Option<Rc<RefCell<RowItem>>> {
    let mut current = root.clone();
    for name in names {
        let next = find_child_by_name(&current, name)?;
        current = next;
    }
    Some(current)
}

#[cfg(test)]
pub(super) fn insert_grandchild_into_parent(
    parent_child: &Rc<RefCell<RowItem>>,
    grandchild_row: Rc<RefCell<RowItem>>,
    child_size: u64,
    child_descendant_count: usize,
) {
    let mut parent_ref = parent_child.borrow_mut();
    let parent_size = parent_ref.size.get_value();
    parent_ref.size = Size::new(parent_size + child_size);
    parent_ref.descendant_count += 1 + child_descendant_count;
    parent_ref.has_children = true;

    let gc_size_val = grandchild_row.borrow().size.get_value();
    let gc_name = grandchild_row.borrow().path_segment.clone();
    let insert_pos = find_sorted_insert_position(&parent_ref.children, gc_size_val, &gc_name);
    parent_ref.children.insert(insert_pos, grandchild_row);

    // Update parent prefix when first grandchild is added.
    if parent_ref.children.len() == 1 {
        let current_prefix = parent_ref.tree_prefix.clone();
        // Replace trailing leaf marker with branch marker.
        if current_prefix.ends_with("──") {
            parent_ref.tree_prefix =
                format!("{}─┬", &current_prefix[..current_prefix.len() - "──".len()]);
        }
    }

    let parent_prefix = parent_ref.tree_prefix.clone();
    let parent_prefix_for_children = build_child_prefix_base(&parent_prefix);
    parent_ref.update_inserted_child_prefix(insert_pos, &parent_prefix_for_children);
}

#[cfg(test)]
pub(super) fn update_root_for_grandchild(
    root: &Rc<RefCell<RowItem>>,
    parent_name: &str,
    child_size: u64,
    child_descendant_count: usize,
) {
    let mut root_ref = root.borrow_mut();
    let root_size = root_ref.size.get_value();
    root_ref.size = Size::new(root_size + child_size);
    root_ref.descendant_count += 1 + child_descendant_count;
    resort_child_in_parent(&mut root_ref, parent_name, " ");
}

/// Returns the index at which a child with `size` and `name` should be inserted
/// into `children` to maintain descending-by-size, ascending-by-name order.
fn find_sorted_insert_position(children: &[Rc<RefCell<RowItem>>], size: u64, name: &str) -> usize {
    children
        .binary_search_by(|existing| {
            let existing_size = existing.borrow().size.get_value();
            match size.cmp(&existing_size) {
                std::cmp::Ordering::Equal => {
                    let existing_name = existing.borrow().path_segment.clone();
                    existing_name.as_str().cmp(name)
                }
                ord => ord,
            }
        })
        .unwrap_or_else(|pos| pos)
}

/// Re-sorts a child named `child_name` within `parent_ref.children` after its
/// size has changed, and updates the affected tree prefixes.
pub(in crate::cli) fn resort_child_in_parent(
    parent_ref: &mut RowItem,
    child_name: &str,
    prefix_for_children: &str,
) {
    let Some(old_index) = parent_ref
        .children
        .iter()
        .position(|c| c.borrow().path_segment == child_name)
    else {
        return;
    };

    let original_len = parent_ref.children.len();
    let child_rc = parent_ref.children.remove(old_index);
    let new_size = child_rc.borrow().size.get_value();
    let new_name = child_rc.borrow().path_segment.clone();
    let new_index = find_sorted_insert_position(&parent_ref.children, new_size, &new_name);
    parent_ref.children.insert(new_index, child_rc);

    if new_index == old_index {
        return; // Position unchanged; no prefix update needed.
    }

    // Update the re-inserted child and, if it's now last, demote the previous last.
    parent_ref.update_inserted_child_prefix(new_index, prefix_for_children);

    // If the child was previously last and moved to an earlier position, the item
    // now occupying the last slot needs its "last child" prefix applied.
    if old_index == original_len - 1 {
        let last_index = parent_ref.children.len() - 1;
        parent_ref.update_inserted_child_prefix(last_index, prefix_for_children);
    }
}

/// Derives the prefix string to pass to `update_inserted_child_prefix` for a
/// parent's children, based on the parent's own `tree_prefix`.
///
/// The parent's `tree_prefix` is structured as `[ancestor columns][self marker]`.
/// For a last child the self marker is `└─┬` (or `└──` for leaves), and for a
/// non-last child it is `─┬` (or `──`).  Children need the ancestor columns
/// plus a continuation column for the parent's own level:
///   - last parent (`└─┬`): strip 3 chars, add 2 spaces (nothing continues)
///   - non-last parent (`─┬`): strip 2 chars, add 1 space (the `├` that the
///     caller appends will later be converted to `│` by `update_tree_prefix`)
pub(in crate::cli) fn build_child_prefix_base(parent_prefix: &str) -> String {
    if parent_prefix.ends_with("└─┬") || parent_prefix.ends_with("└──") {
        // Last child: └─ occupies the continuation column, replaced by spaces.
        let ancestor = &parent_prefix[..parent_prefix.len() - "└─┬".len()];
        format!("{ancestor}  ")
    } else if parent_prefix.ends_with("─┬") || parent_prefix.ends_with("──") {
        // Non-last child: ─ before ┬/─ is the branch connector, replaced by a space.
        let ancestor = &parent_prefix[..parent_prefix.len() - "─┬".len()];
        format!("{ancestor} ")
    } else {
        format!("{parent_prefix} ")
    }
}
