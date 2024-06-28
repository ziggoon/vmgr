use ratatui::{
    layout::Constraint,
    prelude::*,
    style::Style,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        Table, Wrap,
    },
    Frame,
};

use crate::app::App;

const INFO_TEXT: &str =
    "(q) quit | (↑) move up | (↓) move down | (x) start / stop vm | (s) snapshot vm";

pub fn render(f: &mut Frame, app: &mut App) {
    let layout =
        Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]).split(f.size());
    let upper_layout = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Percentage(70),
        Constraint::Min(1),
    ])
    .split(layout[0]);
    let table_layout =
        Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(layout[1]);

    render_overview(f, app, upper_layout[1]);
    render_table(f, app, table_layout[0]);
    render_scrollbar(f, app, table_layout[0]);
    render_footer(f, app, table_layout[1]);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);

    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["id", "name", "cpu usage", "memory usage", "status"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let rows = app.table_data.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };

        let item = data.ref_array();
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(4)
    });
    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            Constraint::Length(app.max_item_lens.0 + 1),
            Constraint::Min(app.max_item_lens.1 + 1),
            Constraint::Min(app.max_item_lens.2 + 1),
            Constraint::Min(app.max_item_lens.3 + 1),
            Constraint::Min(app.max_item_lens.4 + 1),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(t, area, &mut app.table_state)
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
}

fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let index = app.table_state.selected().unwrap();
    let overview = Paragraph::new(vec![
        Line::from(format!("Name: {}", app.table_data[index].name)),
        Line::from(format!("Status: {}", app.table_data[index].status)),
        Line::from(format!("CPU Usage: {}", app.table_data[index].cpu_usage)),
        Line::from(format!("Mem Usage: {}", app.table_data[index].mem_usage)),
        Line::from(format!("Network: {}", app.metrics[index].net_name)),
        Line::from(format!(
            "MB upload: {:.2}",
            app.metrics[index].net_rx as f64 / 1024.0
        )),
        Line::from(format!(
            "MB download: {:.2}",
            app.metrics[index].net_tx as f64 / 1024.0
        )),
        Line::from(format!("Disk: {}", app.metrics[index].disk_name)),
        Line::from(format!("path: {}", app.metrics[index].disk_path)),
        Line::from(format!(
            "MB read: {}",
            app.metrics[index].disk_rx as f64 / 1024.0
        )),
        Line::from(format!(
            "MB written: {}",
            app.metrics[index].disk_wx as f64 / 1024.0
        )),
    ])
    .block(
        Block::bordered()
            .title("VM statistics")
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(app.colors.footer_border_color)),
    )
    .wrap(Wrap { trim: true });

    f.render_widget(overview, area);
}
