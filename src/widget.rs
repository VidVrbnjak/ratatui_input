use crate::InputState;
use ratatui::prelude::*;

/// Input widget
#[derive(Debug, Clone)]
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
        let cursor_char_index = state.cursor_char_idx();
        let selection_char_range = state.selection().map(|f| f.char_range);
        let view_window = &mut state.view_window;
        let old_width = view_window.width;

        view_window.width = area.width as usize;
        if view_window.width > old_width {
            // Increase view window width to the left, the remaining increase is added to the right
            let left_increase = if view_window.offsett < view_window.width - old_width {
                view_window.offsett
            } else {
                view_window.width - old_width
            };

            view_window.offsett -= left_increase;
        };

        if view_window.width < old_width {
            // Shrink view window, so that the cursor falls to the very right of the view window,
            // the remaining width is then shruk from the left
            let right_shrink = view_window.offsett + old_width - cursor_char_index;
            let left_shirnk = view_window.width - right_shrink;

            view_window.offsett += left_shirnk;
            view_window.offsett -= right_shrink;
        };

        let view_window = view_window.clone();

        let mut text_iter = state
            .text()
            .chars()
            .skip(view_window.offsett)
            .take(view_window.width);

        for idx in 0..view_window.width {
            let cell = buf.get_mut(area.x + idx as u16, area.y);
            let symbol = if state.view_window.contains(idx + view_window.offsett) {
                text_iter.next().unwrap_or(' ')
            } else {
                ' '
            };

            if selection_char_range
                .as_ref()
                .is_some_and(|cr| cr.contains(&(idx + view_window.offsett)))
                || cursor_char_index == idx + view_window.offsett
            {
                // This is a highlighted cell, becuase the cursor or selection is on it
                let _ = cell
                    .set_bg(self.fg)
                    .set_fg(self.bg)
                    .set_symbol(symbol.to_string().as_str());
            } else {
                // Normal cell
                let _ = cell
                    .set_bg(self.bg)
                    .set_fg(self.fg)
                    .set_symbol(symbol.to_string().as_str());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use ratatui::assert_buffer_eq;

    use super::*;
    use crate::{Message, ViewWindow};

    fn new_buffer(
        content: &str,
        highligh: Range<usize>,
        area: Rect,
        bg: Color,
        fg: Color,
    ) -> Buffer {
        let mut buf = Buffer::empty(area);
        for (idx, char) in content.chars().enumerate() {
            let cell = buf.get_mut(area.x + idx as u16, area.y);
            let _ = cell.set_symbol(char.to_string().as_str());
            if highligh.contains(&idx) {
                cell.bg = fg;
                cell.fg = bg;
            } else {
                cell.bg = bg;
                cell.fg = fg;
            }
        }

        buf
    }

    #[test]
    fn view_window_increase() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        let widget = Input::new();
        let mut state = InputState::default();

        widget.clone().render(buf.area, &mut buf, &mut state);

        assert_eq!(
            state.view_window,
            ViewWindow {
                width: buf.area.width as usize,
                offsett: 0
            }
        )
    }

    #[test]
    fn cursor_highlight() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 3));
        let widget = Input::new();
        let mut state = InputState::default();

        widget.clone().render(buf.area, &mut buf, &mut state);

        let cursor_cell = buf.get(
            (state.cursor_char_idx() - state.view_window.offsett) as u16,
            0,
        );
        assert_eq!(cursor_cell.bg, widget.fg);
        assert_eq!(cursor_cell.fg, widget.bg);
    }

    #[test]
    fn autoscroll_moving_right_on_paste() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let widget = Input::new();
        let mut state = InputState::default();

        state.handle_message(Message::Paste(String::from("foo bar")));

        widget.clone().render(buf.area, &mut buf, &mut state);

        assert_eq!(state.text(), "foo bar");
        assert_eq!(
            state.view_window,
            ViewWindow {
                width: buf.area.width.into(),
                offsett: 3
            }
        );
        assert_buffer_eq!(
            buf,
            new_buffer(
                " bar ",
                state.view_window.into(),
                buf.area,
                widget.bg,
                widget.fg
            )
        );
    }

    #[test]
    fn autoscrolling_moving_right_on_input() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let widget = Input::new();
        let mut state = InputState::default();

        state.handle_message(Message::Char('f'));
        state.handle_message(Message::Char('o'));
        state.handle_message(Message::Char('o'));
        state.handle_message(Message::Char(' '));
        state.handle_message(Message::Char('b'));
        state.handle_message(Message::Char('a'));
        state.handle_message(Message::Char('r'));

        widget.clone().render(buf.area, &mut buf, &mut state);

        assert_eq!(state.text(), "foo bar");
        assert_eq!(state.cursor_char_idx(), 7);
        assert_eq!(
            state.view_window,
            ViewWindow {
                width: 5,
                offsett: 3
            }
        );
        assert_buffer_eq!(
            buf,
            new_buffer(
                " bar ",
                state.view_window.into(),
                buf.area,
                widget.bg,
                widget.fg
            )
        )
    }
}
