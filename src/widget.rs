use std::ops::Range;

use crate::InputState;
use ratatui::prelude::*;

/// Input widget
#[derive(Debug, Clone)]
pub struct Input {
    /// Color of text foreground
    pub text_fg: Color,
    /// Color of text background
    pub text_bg: Color,
    /// Color of cursor and selection foreground
    pub cursor_fg: Color,
    /// Color of cursor and selection background
    pub cursor_bg: Color,
    /// Symbol used to mask the input. Commonly used for passwords
    pub mask_symbol: Option<char>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            text_fg: Color::White,
            text_bg: Color::Black,
            cursor_fg: Color::Black,
            cursor_bg: Color::White,
            mask_symbol: None,
        }
    }
}

impl StatefulWidget for Input {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let cursor_char_index = state.cursor_char_idx();
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

        let mut display_text = state
            .text()
            .to_string()
            .chars()
            .skip(view_window.offsett)
            .take(view_window.width)
            .map(|ch| match self.mask_symbol {
                Some(mask) => mask,
                None => ch,
            })
            .collect::<String>();

        for _ in display_text.chars().count()..(view_window.width) {
            display_text.push(' ');
        }

        let highlight_range = state
            .selection()
            .map_or(Range::default(), |selection| selection.char_range);

        for (idx, symbol) in display_text.chars().enumerate() {
            let cell = buf
                .get_mut(area.x + idx as u16, area.y)
                .set_symbol(symbol.to_string().as_str());

            let _ = if highlight_range.contains(&(view_window.offsett + idx))
                || state.cursor_char_idx() == view_window.offsett + idx
            {
                cell.set_fg(self.cursor_fg).set_bg(self.cursor_bg)
            } else {
                cell.set_fg(self.text_fg).set_bg(self.text_bg)
            };
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
        highlight: Option<Range<usize>>,
        cursor_idx: usize,
        area: Rect,
        bg: Color,
        fg: Color,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(area.x, 0, area.width, 1));
        for (idx, char) in content.chars().enumerate() {
            let cell = buf.get_mut(area.x + idx as u16, 0);
            let cell = cell.set_symbol(char.to_string().as_str());
            if highlight
                .as_ref()
                .is_some_and(|highlight| highlight.contains(&idx))
                || idx == cursor_idx
            {
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
        let widget = Input::default();
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
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let widget = Input::default();
        let mut state = InputState::default();

        widget.clone().render(buf.area, &mut buf, &mut state);

        assert_buffer_eq!(
            buf,
            new_buffer("     ", None, 0, buf.area, widget.text_bg, widget.text_fg)
        )
    }

    #[test]
    fn autoscroll_moving_right_on_paste() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let widget = Input::default();
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
            new_buffer(" bar ", None, 4, buf.area, widget.text_bg, widget.text_fg)
        );
    }

    #[test]
    fn autoscrolling_moving_right_on_input() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let widget = Input::default();
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
            new_buffer(" bar ", None, 4, buf.area, widget.text_bg, widget.text_fg)
        )
    }
}
