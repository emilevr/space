use super::{table_rows, ViewState};
use ratatui::widgets::Row;

impl ViewState {
    pub(crate) fn update_visible_rows(&mut self) -> Vec<Row<'_>> {
        if !self.visible_rows_dirty {
            return self.rebuild_rows_from_cache();
        }

        let mut rows = vec![];
        self.visible_row_items.clear();

        let mut row_index = 0;
        let mut added_count = 0;
        let mut displayable_item_count = 0;

        for item in &self.item_tree {
            table_rows::add_table_row(
                &mut rows,
                &mut self.visible_row_items,
                item.clone(),
                self.size_display_format,
                self.size_threshold_fraction,
                self.visible_offset,
                self.visible_height,
                &mut row_index,
                &mut added_count,
                &mut displayable_item_count,
                &self.skin,
                self.spinner_tick,
                self.table_selected_index,
                0,
            );
        }

        self.displayable_item_count = displayable_item_count;
        self.visible_rows_dirty = false;

        rows
    }

    fn rebuild_rows_from_cache(&self) -> Vec<Row<'_>> {
        self.visible_row_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.table_selected_index;
                let depth = item.borrow().depth;
                let cells = table_rows::get_row_cell_content(
                    item,
                    self.size_display_format,
                    &self.skin,
                    self.spinner_tick,
                    is_selected,
                    depth,
                );
                Row::new(cells).height(1)
            })
            .collect()
    }
}
