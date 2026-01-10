pub mod tree;

use crate::app::App;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    app.clear_old_status();
    tree::render(frame, app);
}
