pub mod preview;
pub mod tree;

use crate::app::App;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    app.clear_old_status();
    tree::render(frame, app);

    // Render preview overlay if active
    if app.show_preview {
        if let Some(preview_data) = app.get_cached_preview() {
            preview::render_preview_overlay(frame, app, preview_data);
        }
    }
}
