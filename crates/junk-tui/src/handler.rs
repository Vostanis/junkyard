use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppResult};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // exit app with 'q' or '<ESC>'
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }

        // 'ENTER' -> search bar
        KeyCode::Enter => app.search_bar(),

        // 'RIGHT' or 'TAB' -> increase tab
        KeyCode::Right | KeyCode::Tab => {
            app.incr_tab();
        }

        // 'LEFT' or 'SHIFT + TAB' -> decrease tab
        KeyCode::Left | KeyCode::BackTab => {
            app.decr_tab();
        }

        // '1' -> stock dashboard
        KeyCode::Char('1') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                // app.stock_dashboard()
            }
        }

        // '2' -> crypto dashboard
        KeyCode::Char('2') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                // app.crypto_dashboard()
            }
        }

        // '0' -> asset screener
        KeyCode::Char('0') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                // app.screener_dashboard()
            }
        }

        _ => {}
    }

    Ok(())
}
