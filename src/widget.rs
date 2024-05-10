use crate::InputState;
use ratatui::prelude::*;

/// Input widget
#[derive(Debug)]
pub struct Input {
    fg: Color,
    bg: Color,
}

impl Input {
    /// Create a new [`Input`] widget
    pub fn new() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
        }
    }
}

impl StatefulWidget for Input {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for (char_idx, char) in state.text().chars().enumerate() {
            let _ = buf
                .get_mut(area.x + char_idx as u16, area.y)
                .set_char(char)
                .set_fg(self.fg)
                .set_bg(self.bg);
        }

        // Last cell outside the value needs to be clear, because we can put the cursor there
        let _ = buf
            .get_mut(area.x + state.text().chars().count() as u16, area.y)
            .set_char(' ');

        // Selection highlight
        if let Some(selection_range) = state.selection_char_range() {
            for idx in selection_range {
                let _ = buf
                    .get_mut(area.x + idx as u16, area.y)
                    .set_fg(self.bg)
                    .set_bg(self.fg);
            }
        }

        // Cursor highlight
        let _ = buf
            .get_mut(area.x + state.cursor_char_idx() as u16, area.y)
            .set_fg(self.bg)
            .set_bg(self.fg);
    }
}
