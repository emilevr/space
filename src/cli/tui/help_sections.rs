use crate::cli::tui::rendering::remove_area_top;
use crate::cli::tui::{
    COLLAPSE_KEY_SYMBOL, COLLAPSE_SELECTED_CHILDREN_KEY, CONFIRM_DELETE_KEY, DELETE_KEY,
    EXPAND_KEY_SYMBOL, EXPAND_SELECTED_CHILDREN_KEY, FILTER_KEY, QUIT_KEY_1, QUIT_KEY_2_SYMBOL,
    RESCAN_KEY, SELECT_FIRST_KEY_SYMBOL, SELECT_LAST_KEY_SYMBOL, SELECT_NEXT_KEY_SYMBOL,
    SELECT_NEXT_PAGE_KEY_SYMBOL, SELECT_PREV_KEY_SYMBOL, SELECT_PREV_PAGE_KEY_SYMBOL,
    VIEW_SIZE_THRESHOLD_0_PERCENT_KEY, VIEW_SIZE_THRESHOLD_10_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_20_PERCENT_KEY, VIEW_SIZE_THRESHOLD_30_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_40_PERCENT_KEY, VIEW_SIZE_THRESHOLD_50_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_60_PERCENT_KEY, VIEW_SIZE_THRESHOLD_70_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_80_PERCENT_KEY, VIEW_SIZE_THRESHOLD_90_PERCENT_KEY,
};
use ratatui::{
    layout::Constraint,
    prelude::*,
    style::Style,
    widgets::{Block, Cell, Row, Table},
    Frame,
};

pub(super) fn render_general_section<B: Backend>(
    key_style: Style,
    danger_key_style: Style,
    section_header_style: Style,
    key_column_size: usize,
    column_constraints: [Constraint; 3],
    f: &mut Frame<'_, B>,
    area: &mut Rect,
) {
    render_help_section(
        "General",
        vec![
            Row::new(vec![
                Cell::from(format!("{QUIT_KEY_1:^key_column_size$}")).style(key_style),
                Cell::from(format!("{QUIT_KEY_2_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from("Quit the application"),
            ]),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!("{DELETE_KEY:^key_column_size$}")).style(danger_key_style),
                Cell::from(""),
                Cell::from("Delete selected item"),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(format!("{CONFIRM_DELETE_KEY:^key_column_size$}"))
                    .style(danger_key_style),
                Cell::from("Confirm delete (on delete dialog)"),
            ]),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!("{RESCAN_KEY:^key_column_size$}")).style(key_style),
                Cell::from(format!("{:^key_column_size$}", "F5")).style(key_style),
                Cell::from("Rescan selected directory"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        area,
    );
}

pub(super) fn render_navigation_section<B: Backend>(
    key_style: Style,
    section_header_style: Style,
    key_column_size: usize,
    column_constraints: [Constraint; 3],
    f: &mut Frame<'_, B>,
    area: &mut Rect,
) {
    render_help_section(
        "Navigation",
        vec![
            Row::new(vec![
                Cell::from(format!("{SELECT_PREV_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select previous item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_NEXT_KEY_SYMBOL:^key_column_size$}",)).style(key_style),
                Cell::from(""),
                Cell::from("Select next item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_PREV_PAGE_KEY_SYMBOL:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("Select corresponding item on previous page"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_NEXT_PAGE_KEY_SYMBOL:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("Select corresponding item on next page item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_FIRST_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select first item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_LAST_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select last item"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        area,
    );
}

pub(super) fn render_filtering_section<B: Backend>(
    key_style: Style,
    section_header_style: Style,
    key_column_size: usize,
    column_constraints: [Constraint; 3],
    f: &mut Frame<'_, B>,
    area: &mut Rect,
) {
    render_help_section(
        "Filtering",
        vec![
            Row::new(vec![
                Cell::from(format!("{FILTER_KEY:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Enter regex path filter (empty Enter clears)"),
            ]),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!("{COLLAPSE_KEY_SYMBOL:^key_column_size$}",)).style(key_style),
                Cell::from(""),
                Cell::from("Collapse selected / go to parent"),
            ]),
            Row::new(vec![
                Cell::from(format!("{EXPAND_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Expand selected directory item"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{COLLAPSE_SELECTED_CHILDREN_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("If all children are collapsed, then collapse the selected item\nIf one or more children are expanded, then collapse all children of the selected item"),
            ])
            .height(2),
            Row::new(vec![
                Cell::from(format!("{EXPAND_SELECTED_CHILDREN_KEY:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("If selected item is collapsed, then expand and show collapsed children\nIf selected directory item is expanded, then expand all descendants"),
            ])
            .height(2),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_0_PERCENT_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to  0%, i.e. do not hide any items"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_10_PERCENT_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 10%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_20_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 20%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_30_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 30%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_40_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 40%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_50_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 50%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_60_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 60%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_70_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 70%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_80_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 80%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_90_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 90%"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        area,
    );
}

fn render_help_section<B: Backend>(
    title: &str,
    rows: Vec<Row<'_>>,
    title_style: Style,
    column_constraints: [Constraint; 3],
    f: &mut Frame<'_, B>,
    area: &mut Rect,
) {
    let row_count = rows.len() as u16;
    let table = Table::new(rows)
        .block(Block::default().title(title).title_style(title_style))
        .widths(&column_constraints)
        .column_spacing(1);
    f.render_widget(table, *area);
    remove_area_top(area, row_count + 2);
}
