use super::ViewState;
use std::cmp::min;

impl ViewState {
    pub(crate) fn previous(&mut self, increment: usize) {
        let old_offset = self.visible_offset;
        let mut i = self.table_selected_index;
        if i >= increment {
            i -= increment;
        } else if self.visible_offset > increment {
            self.visible_offset -= increment;
            i = 0;
        } else {
            if self.visible_offset + i >= increment {
                self.visible_offset = self.visible_offset + i - increment;
            } else {
                self.visible_offset = 0;
            }
            i = 0;
        }
        self.table_selected_index = i;
        if self.visible_offset != old_offset {
            self.visible_rows_dirty = true;
        }
    }

    pub(crate) fn next(&mut self, increment: usize) {
        if self.displayable_item_count == 0 {
            return;
        }

        let old_offset = self.visible_offset;
        let mut i = self.table_selected_index + increment;

        let max_table_selected_index = self.visible_height - 1;
        if i > max_table_selected_index {
            self.visible_offset += i - max_table_selected_index;
            i = max_table_selected_index;
        }

        if self.visible_offset + i >= self.displayable_item_count {
            self.visible_offset =
                self.displayable_item_count - min(self.visible_height, self.displayable_item_count);
            i = min(max_table_selected_index, self.displayable_item_count - 1);
        }

        self.table_selected_index = i;
        if self.visible_offset != old_offset {
            self.visible_rows_dirty = true;
        }
    }

    pub(crate) fn first(&mut self) {
        self.visible_offset = 0;
        self.table_selected_index = 0;
        self.visible_rows_dirty = true;
    }

    pub(crate) fn last(&mut self) {
        if self.displayable_item_count == 0 {
            return;
        }

        self.visible_offset =
            self.displayable_item_count - min(self.visible_height, self.displayable_item_count);
        self.table_selected_index = min(self.visible_height - 1, self.displayable_item_count - 1);
        self.visible_rows_dirty = true;
    }
}
