use std::error;

use ratatui::prelude::Color;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{ScrollbarState, TableState};

use unicode_width::UnicodeWidthStr;
use virt::connect::Connect;

use crate::vms::*;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const ITEM_HEIGHT: usize = 4;

#[derive(Debug)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(Debug)]
pub struct TableData {
    pub id: String,
    pub name: String,
    pub cpu_usage: String,
    pub mem_usage: String,
    pub status: String,
}

impl TableData {
    pub const fn ref_array(&self) -> [&String; 5] {
        [
            &self.id,
            &self.name,
            &self.cpu_usage,
            &self.mem_usage,
            &self.status,
        ]
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn cpu_usage(&self) -> &str {
        &self.cpu_usage
    }

    fn mem_usage(&self) -> &str {
        &self.mem_usage
    }

    fn status(&self) -> &str {
        &self.status
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub conn: Connect,
    pub table_state: TableState,
    pub max_item_lens: (u16, u16, u16, u16, u16),
    pub scroll_state: ScrollbarState,
    pub colors: TableColors,
    pub metrics: Vec<VmMetrics>,
    pub table_data: Vec<TableData>,
}

impl Default for App {
    fn default() -> Self {
        let conn: Connect = connect("qemu:///system");
        let mut table_data: Vec<TableData> = vec![];
        let metrics: Vec<VmMetrics> = get_vm_data(&conn);

        for domain in &metrics {
            table_data.push(TableData {
                id: domain.id.to_string(),
                name: domain.name.clone(),
                cpu_usage: 0.to_string(),
                mem_usage: format!("{}", domain.mem_rss + domain.mem_cache),
                status: if domain.status == true {
                    String::from("on")
                } else {
                    String::from("off")
                },
            });
        }

        Self {
            running: true,
            conn,
            table_state: TableState::default().with_selected(0),
            max_item_lens: constraint_len_calculator(&table_data),
            scroll_state: ScrollbarState::new((table_data.len() - 1) * ITEM_HEIGHT),
            colors: TableColors::new(&PALETTES[0]),
            metrics,
            table_data,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        return Self::default();
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        let mut table_data: Vec<TableData> = vec![];
        let metrics: Vec<VmMetrics> = get_vm_data(&self.conn);

        for (i, domain) in metrics.iter().enumerate() {
            let elapsed = domain
                .timestamp
                .duration_since(self.metrics[i].timestamp)
                .as_secs_f64();
            table_data.push(TableData {
                id: domain.id.to_string(),
                name: domain.name.clone(),
                cpu_usage: if elapsed > 0.0 {
                    let time_diff = domain.cpu_time.saturating_sub(self.metrics[i].cpu_time) as f64
                        / 1_000_000_000.0;
                    format!("{:.2}%", (time_diff as f64 / elapsed) * 100.0)
                } else {
                    format!("{:.2}%", 0.0)
                },
                mem_usage: format!("{} Mb", (domain.mem_rss + domain.mem_cache) / 1024),
                status: if domain.status == true {
                    String::from("on")
                } else {
                    String::from("off")
                },
            })
        }

        self.table_data = table_data;
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.table_data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn prev(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.table_data.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        disconnect(&mut self.conn);
        self.running = false;
    }
}

fn constraint_len_calculator(items: &[TableData]) -> (u16, u16, u16, u16, u16) {
    let id_len = items
        .iter()
        .map(TableData::id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let name_len = items
        .iter()
        .map(TableData::name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let cpu_len = items
        .iter()
        .map(TableData::cpu_usage)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let mem_len = items
        .iter()
        .map(TableData::mem_usage)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let status_len = items
        .iter()
        .map(TableData::status)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    return (
        id_len as u16,
        name_len as u16,
        cpu_len as u16,
        mem_len as u16,
        status_len as u16,
    );
}
