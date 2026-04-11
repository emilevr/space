use ratatui::prelude::*;
pub(in crate::cli::tui) fn expand_area(area: &mut Rect, cx: u16, cy: u16, area_limit: Rect) {
    if area.x >= cx {
        area.x -= cx;
    } else {
        area.x = 0;
    }

    if area.y >= cy {
        area.y -= cy;
    } else {
        area.y = 0;
    }

    area.width += cx * 2;
    area.height += cy * 2;

    *area = area.intersection(area_limit);
}

pub(in crate::cli::tui) fn contract_area(rect: &mut Rect, cx: u16, cy: u16) {
    rect.x += cx;
    rect.y += cy;

    if rect.width >= cx * 2 {
        rect.width -= cx * 2;
    } else {
        rect.width = 0;
    }

    if rect.height >= cy * 2 {
        rect.height -= cy * 2;
    } else {
        rect.height = 0;
    }
}

pub(in crate::cli::tui) fn remove_area_top(rect: &mut Rect, by_y: u16) {
    rect.y += by_y;
    if rect.height >= by_y {
        rect.height -= by_y;
    } else {
        rect.height = 0;
    }
}
