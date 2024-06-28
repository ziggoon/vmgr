use crate::{
    app::{App, AppResult},
    vms::{snapshot, start, stop},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Counter handlers
        KeyCode::Up => {
            app.prev();
        }
        KeyCode::Down => {
            app.next();
        }
        KeyCode::Char('x') => {
            let current_item = &app.table_data[app.table_state.selected().unwrap()];
            let name = &current_item.name;
            let status = &current_item.status;

            if status == "off" {
                start(&app.conn, name);
            } else {
                stop(&app.conn, name);
            }
        }
        KeyCode::Char('s') => {
            let current_item = &app.table_data[app.table_state.selected().unwrap()];
            let name = &current_item.name;

            snapshot(&app.conn, name);
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
