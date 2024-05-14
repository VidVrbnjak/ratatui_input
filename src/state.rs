use std::{
    cmp::{max, min},
    ops::{Deref, Range},
};

use clipboard::ClipboardProvider;

use crate::Message;

/// Stored state of the input widget. Used for the cursor position, text selection and windowing/scrolling
#[derive(Debug, PartialEq, Eq)]
pub struct InputState {
    value: String,
    cursor_char_idx: usize,
    in_focus: bool,
    insert_mode: bool,
    selection_start_char_idx: Option<usize>,
    pub(crate) view_window: ViewWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ViewWindow {
    /// Width of the window
    pub(crate) width: usize,
    /// Offsett from the left for the window
    pub(crate) offsett: usize,
}

impl ViewWindow {
    pub fn contains(&self, idx: usize) -> bool {
        self.offsett <= idx && self.offsett + self.width > idx
    }
}

impl From<Range<usize>> for ViewWindow {
    fn from(value: Range<usize>) -> Self {
        ViewWindow {
            offsett: value.start,
            width: value.len(),
        }
    }
}

impl From<ViewWindow> for Range<usize> {
    fn from(val: ViewWindow) -> Self {
        val.offsett..(val.offsett + val.width)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_char_idx: 0,
            in_focus: false,
            insert_mode: false,
            selection_start_char_idx: None,
            view_window: ViewWindow {
                width: 1,
                offsett: 0,
            },
        }
    }
}

impl InputState {
    /// Update the [`InputState`] with the given message
    pub fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Empty => {}
            Message::Focus => self.in_focus = true,
            Message::RemoveFocus => self.in_focus = false,
            Message::DeleteOnCursor => {
                match self.selection() {
                    Some(selection) => {
                        // Replace the whole selection with a empty string
                        self.value.replace_range(selection.byte_range.clone(), "");

                        // Keep the cursor on the same character (not the same index)
                        self.cursor_char_idx = selection.char_range.start;

                        // End the selection
                        self.selection_start_char_idx = None;

                        // Cursor might fall outside the view window, so we move it to the left as needed
                        self.view_window.offsett =
                            min(self.view_window.offsett, self.cursor_char_idx);
                    }
                    None => {
                        if self.cursor_char_idx == self.value.chars().count() {
                            // Do nothing because we are not currently on a character
                        } else {
                            // Remove the character from the string
                            let idx = char_idx_to_byte_idx(&self.value, self.cursor_char_idx);
                            let _ = self.value.remove(idx);

                            // Cursor might fall outside the view window, so we move it to the left as needed
                            self.view_window.offsett =
                                min(self.view_window.offsett, self.cursor_char_idx);
                        }
                    }
                }
            }
            Message::DeleteBeforeCursor => {
                match self.selection() {
                    Some(selection) => {
                        // Replace the selection with a empty string
                        self.value.replace_range(selection.byte_range, "");

                        // Keep the cursor on same character (not character index)
                        self.cursor_char_idx = selection.char_range.start;

                        // End selection
                        self.selection_start_char_idx = None;

                        // Cursor might fall outside the view window, so we move it to the left as needed
                        self.view_window.offsett =
                            min(self.view_window.offsett, self.cursor_char_idx);
                    }
                    None => {
                        if self.cursor_char_idx == 0 {
                            // Do nothing because we are at the start
                        } else {
                            let idx = self
                                .value
                                .char_indices()
                                .enumerate()
                                .find(|(char_idx, _)| char_idx == &(self.cursor_char_idx - 1))
                                .map(|(_, (byte_idx, _))| byte_idx)
                                .unwrap();

                            let _ = self.value.remove(idx);
                            self.cursor_char_idx -= 1;

                            // Cursor might fall outside the view window, so we move it to the left as needed
                            self.view_window.offsett =
                                min(self.view_window.offsett, self.cursor_char_idx);
                        }
                    }
                }
            }
            Message::MoveLeft => {
                // End selection
                self.selection_start_char_idx = None;

                if self.cursor_char_idx == 0 {
                    // We are at the start already so we cannot move anymore
                } else {
                    self.cursor_char_idx -= 1;
                    self.view_window.offsett = min(self.view_window.offsett, self.cursor_char_idx);
                }
            }
            Message::MoveRight => {
                // End selection
                self.selection_start_char_idx = None;

                if self.cursor_char_idx == self.value.chars().count() {
                    // We are already 1 step ahead of the value, so we cannot move anymore
                } else {
                    self.cursor_char_idx += 1;
                    if !self.view_window.contains(self.cursor_char_idx) {
                        self.view_window.offsett =
                            self.cursor_char_idx + 1 - self.view_window.width;
                    }
                }
            }
            Message::JumpToEnd => {
                self.cursor_char_idx = self.value.chars().count();
                if !self.view_window.contains(self.cursor_char_idx) {
                    self.view_window.offsett = self.cursor_char_idx + 1 - self.view_window.width;
                }
            }
            Message::JumpToStart => {
                self.cursor_char_idx = 0;
                self.view_window.offsett = min(self.view_window.offsett, self.cursor_char_idx);
            }
            Message::Char(c) => {
                match self.selection() {
                    Some(selection) => {
                        // Replace the entire selection with the input
                        self.value
                            .replace_range(selection.byte_range, c.to_string().as_str());
                        self.cursor_char_idx = selection.char_range.start + 1;
                        self.selection_start_char_idx = None;
                        self.view_window.offsett =
                            min(self.view_window.offsett, self.cursor_char_idx);
                    }
                    None => {
                        if self.cursor_char_idx == self.value.chars().count() {
                            // We are outside the string, so we push onto it
                            self.value.push(c);
                        } else if self.insert_mode {
                            // The cursor is on a character so we replace that character
                            let start_idx = self
                                .value
                                .char_indices()
                                .enumerate()
                                .find(|(char_idx, _)| char_idx == &self.cursor_char_idx)
                                .map(|(_, (byte_idx, _))| byte_idx)
                                .unwrap();

                            let end_idx = self
                                .value
                                .char_indices()
                                .enumerate()
                                .find(|(char_idx, _)| char_idx + 1 == self.cursor_char_idx)
                                .map(|(_, (byte_idx, _))| byte_idx)
                                .unwrap_or(self.value.len());

                            self.value.replace_range(
                                start_idx..end_idx,
                                Into::<String>::into(c).as_str(),
                            );
                        } else {
                            // The cursor is on a character inside the string so we insert the character at that position
                            let idx = self
                                .value
                                .char_indices()
                                .enumerate()
                                .find(|(char_idx, _)| char_idx == &self.cursor_char_idx)
                                .map(|(_, (byte_idx, _))| byte_idx)
                                .unwrap();

                            self.value.insert(idx, c);
                        }
                        self.cursor_char_idx += 1;
                        if !self.view_window.contains(self.cursor_char_idx) {
                            self.view_window.offsett =
                                self.cursor_char_idx + 1 - self.view_window.width;
                        }
                    }
                }
            }
            Message::Paste(str) => match self.selection() {
                Some(selection) => {
                    self.value.replace_range(selection.byte_range, &str);
                    self.cursor_char_idx = selection.char_range.end;
                    self.selection_start_char_idx = None;
                    self.view_window.offsett =
                        if selection.text.chars().count() > str.chars().count() {
                            // Replaced text was longer than pasted text, so view window moves left
                            min(self.view_window.offsett, self.cursor_char_idx)
                        } else {
                            // Replaced text was shorter than pasted text, so view window moves right
                            self.cursor_char_idx + 1 - self.view_window.width
                        };
                }
                None => {
                    if self.cursor_char_idx == self.value.chars().count() {
                        self.value.push_str(str.as_str());
                    } else {
                        self.value
                            .insert_str(self.cursor_char_idx() + 1, str.as_str());
                    }
                    self.cursor_char_idx += str.chars().count();
                    if !self.view_window.contains(self.cursor_char_idx) {
                        self.view_window.offsett =
                            self.cursor_char_idx + 1 - self.view_window.width;
                    }
                }
            },
            Message::ToggleInsertMode => self.insert_mode = !self.insert_mode,
            Message::MoveLeftWithSelection => {
                if self.cursor_char_idx == 0 {
                    // We are at the very, start and cannot move anywhere
                } else {
                    self.selection_start_char_idx = match self.selection_start_char_idx {
                        Some(selection_start_char_idx) => {
                            if selection_start_char_idx == self.cursor_char_idx - 1 {
                                // The cursor has come back to the selection start so we have nothing selected anymore
                                None
                            } else {
                                // Start of the selection stays the same
                                Some(selection_start_char_idx)
                            }
                        }
                        None => {
                            // Start selection
                            if self.cursor_char_idx == self.value.chars().count() {
                                Some(self.cursor_char_idx - 1)
                            } else {
                                Some(self.cursor_char_idx)
                            }
                        }
                    };

                    self.cursor_char_idx -= 1;
                    self.view_window.offsett = min(self.view_window.offsett, self.cursor_char_idx);
                }
            }
            Message::MoveRightWithSelection => {
                if self.cursor_char_idx + 1 == self.value.chars().count() {
                    // Cannot move anymore
                } else {
                    self.selection_start_char_idx = match self.selection_start_char_idx {
                        Some(selection_start_char_idx) => {
                            if self.cursor_char_idx + 1 == selection_start_char_idx {
                                // The cursor has come back to the selection start so we have nothing selected anymore
                                None
                            } else {
                                // Start of the selection stays the same
                                Some(selection_start_char_idx)
                            }
                        }
                        None => {
                            // Start selection
                            Some(self.cursor_char_idx)
                        }
                    };

                    self.cursor_char_idx += 1;
                    if !self.view_window.contains(self.cursor_char_idx) {
                        self.view_window.offsett =
                            self.cursor_char_idx + 1 - self.view_window.width;
                    }
                }
            }
            Message::JumpToEndWithSelection => {
                if self.cursor_char_idx == self.value.chars().count() {
                    return;
                }

                if self.selection_start_char_idx.is_none() {
                    self.selection_start_char_idx = Some(self.cursor_char_idx);
                }

                self.cursor_char_idx = self.value.chars().count() - 1;
                if !self.view_window.contains(self.cursor_char_idx) {
                    self.view_window.offsett = self.cursor_char_idx + 1 - self.view_window.width;
                }
            }
            Message::JumpToStartWithSelection => {
                if self.cursor_char_idx == 0 {
                    return;
                }

                if self.selection_start_char_idx.is_none() {
                    if self.cursor_char_idx == self.value.chars().count() {
                        self.selection_start_char_idx = Some(self.cursor_char_idx - 1);
                    } else {
                        self.selection_start_char_idx = Some(self.cursor_char_idx);
                    }
                }

                self.cursor_char_idx = 0;
                self.view_window.offsett = min(self.view_window.offsett, self.cursor_char_idx);
            }
            Message::Copy => match self.selection() {
                Some(selection) => {
                    // Copy the selection
                    let _ = clipboard::ClipboardContext::new()
                        .and_then(|mut cc| cc.set_contents(selection.to_string()));
                }
                None => {
                    // No selection, so we copy the entire value
                    let _ = clipboard::ClipboardContext::new()
                        .and_then(|mut cc| cc.set_contents(self.value.clone()));
                }
            },
            Message::Cut => {
                match self.selection() {
                    Some(selection) => {
                        // Cut the selection and set cursor to the start of the selecion
                        let _ = clipboard::ClipboardContext::new()
                            .and_then(|mut cc| cc.set_contents(selection.to_string()));
                        self.cursor_char_idx = selection.char_range.start;
                        self.selection_start_char_idx = None;
                        let mut taken_iter = (0..self.value.chars().count())
                            .map(|char_idx| selection.char_range.contains(&char_idx));
                        self.value.retain(|_| !taken_iter.next().unwrap());
                    }
                    None => {
                        // Copy the entire value and then clear it
                        let _ = clipboard::ClipboardContext::new()
                            .and_then(|mut cc| cc.set_contents(self.value.clone()));
                        self.value.clear();
                        self.cursor_char_idx = 0;
                        self.selection_start_char_idx = None;
                    }
                };
                self.view_window.offsett = min(self.view_window.offsett, self.cursor_char_idx);
            }
        }
    }

    /// Current value of the input
    pub fn text(&self) -> &str {
        &self.value
    }

    #[allow(unused)]
    pub(crate) fn cursor_byte_idx(&self) -> usize {
        char_idx_to_byte_idx(&self.value, self.cursor_char_idx)
    }

    pub(crate) fn cursor_char_idx(&self) -> usize {
        self.cursor_char_idx
    }

    /// Currently selected text
    pub fn selection(&self) -> Option<Selection> {
        match self.selection_start_char_idx {
            Some(start_char_idx) => {
                let min_char_idx = min(start_char_idx, self.cursor_char_idx);
                let min_byte_idx = char_idx_to_byte_idx(&self.value, min_char_idx);
                let max_char_idx = max(start_char_idx, self.cursor_char_idx) + 1;
                let max_byte_idx = char_idx_to_byte_idx(&self.value, max_char_idx);

                Some(Selection {
                    char_range: min_char_idx..max_char_idx,
                    byte_range: min_byte_idx..max_byte_idx,
                    text: self.value[min_byte_idx..max_byte_idx].to_string(),
                })
            }
            None => None,
        }
    }
}

fn char_idx_to_byte_idx(str: &str, char_idx: usize) -> usize {
    str.char_indices()
        .enumerate()
        .find(|(idx, _)| idx == &char_idx)
        .map(|(_, (idx, _))| idx)
        .unwrap_or(str.len())
}

/// Selected text inside the [`InputState`]
#[derive(Debug)]
pub struct Selection {
    pub(crate) char_range: Range<usize>,
    pub(crate) byte_range: Range<usize>,
    text: String,
}

impl Deref for Selection {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_message() {
        let mut state = InputState::default();
        state.handle_message(Message::Empty);
        assert_eq!(state, InputState::default());
    }

    #[test]
    fn focus() {
        let mut state = InputState::default();
        assert!(!state.in_focus);

        state.handle_message(Message::Focus);
        assert!(state.in_focus);

        state.handle_message(Message::RemoveFocus);
        assert!(!state.in_focus);
    }

    #[test]
    fn delete_on_cursor() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            ..Default::default()
        };

        state.handle_message(Message::DeleteOnCursor);

        assert_eq!(state.text(), "žđščć🎈👓");
        assert_eq!(state.cursor_char_idx(), 6);
    }

    #[test]
    fn delete_on_cursor_with_selection() {
        //žđščć[🎈🎨]👓
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            selection_start_char_idx: Some(5),
            ..Default::default()
        };

        assert_eq!(&*state.selection().unwrap(), "🎈🎨");
        state.handle_message(Message::DeleteOnCursor);

        //žđšč[ć]👓
        assert_eq!(state.text(), "žđščć👓");
        assert_eq!(state.cursor_char_idx(), 5);
    }

    #[test]
    fn delete_before_cursor() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            ..Default::default()
        };

        state.handle_message(Message::DeleteBeforeCursor);

        assert_eq!(state.text(), "žđščć🎨👓");
    }

    #[test]
    fn delete_before_cursor_with_selection() {
        //žđšč[ć🎈🎨]👓
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            selection_start_char_idx: Some(4),
            ..Default::default()
        };

        assert_eq!(&*state.selection().unwrap(), "ć🎈🎨");

        state.handle_message(Message::DeleteBeforeCursor);

        assert_eq!(state.text(), "žđšč👓");
        assert_eq!(state.cursor_char_idx, 4);
    }

    #[test]
    fn move_left() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 7,
            ..Default::default()
        };

        state.handle_message(Message::MoveLeft);
        assert_eq!(state.cursor_char_idx, 6);

        state.handle_message(Message::MoveLeft);
        assert_eq!(state.cursor_char_idx, 5);
    }

    #[test]
    fn move_left_cancles_selection() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 7,
            selection_start_char_idx: Some(5),
            ..Default::default()
        };

        state.handle_message(Message::MoveLeft);
        assert!(state.selection().is_none());
    }

    #[test]
    fn move_left_on_start() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            ..Default::default()
        };

        state.handle_message(Message::MoveLeft);

        assert_eq!(state.cursor_char_idx(), 0);
    }

    #[test]
    fn move_left_with_selection() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            ..Default::default()
        };

        state.handle_message(Message::MoveLeftWithSelection);

        assert_eq!(&*state.selection().unwrap(), "🎈🎨");
        assert_eq!(state.cursor_char_idx(), 5);
    }

    #[test]
    fn move_right() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            ..Default::default()
        };

        state.handle_message(Message::MoveRight);
        assert_eq!(state.cursor_char_idx(), 1);

        state.handle_message(Message::MoveRight);
        assert_eq!(state.cursor_char_idx(), 2);
    }

    #[test]
    fn moving_right_cancles_selection() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            selection_start_char_idx: Some(5),
            ..Default::default()
        };

        state.handle_message(Message::MoveRight);
        assert!(state.selection().is_none());
    }

    #[test]
    fn move_right_on_end() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 8,
            ..Default::default()
        };

        state.handle_message(Message::MoveRight);

        assert_eq!(state.cursor_char_idx(), 8);
    }

    #[test]
    fn move_right_with_selecion() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            ..Default::default()
        };

        state.handle_message(Message::MoveRightWithSelection);

        assert_eq!(&*state.selection().unwrap(), "žđ");
        assert_eq!(state.cursor_char_idx(), 1);
    }

    #[test]
    fn jump_to_end() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            ..Default::default()
        };

        state.handle_message(Message::JumpToEnd);

        assert_eq!(state.cursor_byte_idx(), state.text().len());
    }

    #[test]
    fn jump_to_end_with_selection() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            ..Default::default()
        };

        state.handle_message(Message::JumpToEndWithSelection);

        assert_eq!(&*state.selection().unwrap(), state.text());
        assert_eq!(state.cursor_char_idx(), 7);
    }

    #[test]
    fn jump_to_start() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 7,
            ..Default::default()
        };

        state.handle_message(Message::JumpToStart);

        assert_eq!(state.cursor_char_idx(), 0);
    }

    #[test]
    fn jump_to_start_with_selection() {
        let mut state = InputState {
            value: String::from("žđščć🎈🎨👓"),
            cursor_char_idx: 6,
            ..Default::default()
        };

        state.handle_message(Message::JumpToStartWithSelection);

        assert_eq!(state.cursor_char_idx(), 0);
        assert_eq!(&*state.selection().unwrap(), "žđščć🎈🎨");
    }

    #[test]
    fn character_input_at_end() {
        let mut state = InputState {
            value: String::new(),
            ..Default::default()
        };

        state.handle_message(Message::Char('0'));
        assert_eq!(state.text(), "0");
        assert_eq!(state.cursor_byte_idx(), 1);

        state.handle_message(Message::Char('1'));
        assert_eq!(state.text(), "01");
        assert_eq!(state.cursor_byte_idx(), 2);
    }

    #[test]
    fn character_input_in_middle() {
        let mut state = InputState {
            value: String::from("foo bar"),
            cursor_char_idx: 3,
            ..Default::default()
        };

        state.handle_message(Message::Char(' '));
        state.handle_message(Message::Char('😎'));

        assert_eq!(state.text(), "foo 😎 bar");
        assert_eq!(state.cursor_char_idx(), 5);
    }

    #[test]
    fn character_input_at_end_with_insert_mode() {
        let mut state = InputState {
            value: String::new(),
            insert_mode: true,
            ..Default::default()
        };

        state.handle_message(Message::Char('0'));
        assert_eq!(state.text(), "0");
        assert_eq!(state.cursor_byte_idx(), 1);

        state.handle_message(Message::Char('1'));
        assert_eq!(state.text(), "01");
        assert_eq!(state.cursor_byte_idx(), 2);
    }

    #[test]
    fn character_input_in_insert_mode() {
        let mut state = InputState {
            value: String::from("foo bar"),
            insert_mode: true,
            ..Default::default()
        };

        state.handle_message(Message::Char('h'));
        state.handle_message(Message::Char('e'));
        state.handle_message(Message::Char('l'));
        state.handle_message(Message::Char('l'));
        state.handle_message(Message::Char('o'));
        state.handle_message(Message::Char(' '));
        state.handle_message(Message::Char('w'));
        state.handle_message(Message::Char('o'));
        state.handle_message(Message::Char('r'));
        state.handle_message(Message::Char('l'));
        state.handle_message(Message::Char('d'));

        assert_eq!(state.text(), "hello world");
        assert_eq!(state.cursor_char_idx(), 11);
    }

    #[test]
    fn character_input_on_selection() {
        let mut state = InputState {
            value: String::from("foo bar"),
            cursor_char_idx: 5,
            selection_start_char_idx: Some(1),
            ..Default::default()
        };

        state.handle_message(Message::Char('a'));

        assert_eq!(state.text(), "far");
        assert_eq!(state.cursor_char_idx(), 2);
    }

    #[test]
    fn character_input_at_end_in_insert_mode() {
        let mut state = InputState {
            value: String::new(),
            insert_mode: true,
            ..Default::default()
        };

        state.handle_message(Message::Char('0'));
        assert_eq!(state.text(), "0");
        assert_eq!(state.cursor_byte_idx(), 1);

        state.handle_message(Message::Char('1'));
        assert_eq!(state.text(), "01");
        assert_eq!(state.cursor_byte_idx(), 2);
    }

    #[test]
    fn paste_at_end() {
        let mut state = InputState {
            value: String::from("foo"),
            cursor_char_idx: 3,
            ..Default::default()
        };

        state.handle_message(Message::Paste(String::from(" bar")));

        assert_eq!(state.text(), "foo bar");
        assert_eq!(state.cursor_char_idx(), 7);
    }

    #[test]
    fn paste_in_middle() {
        let mut state = InputState {
            value: String::from("foo bar"),
            cursor_char_idx: 3,
            ..Default::default()
        };

        state.handle_message(Message::Paste(String::from("baz ")));

        assert_eq!(state.text(), "foo baz bar");
        assert_eq!(state.cursor_char_idx(), 7);
    }

    #[test]
    fn paste_on_selection() {
        let mut state = InputState {
            value: String::from("foo bar"),
            cursor_char_idx: 4,
            selection_start_char_idx: Some(2),
            ..Default::default()
        };

        state.handle_message(Message::Paste(String::from("faz")));

        assert_eq!(state.text(), "fofazar");
        assert_eq!(state.cursor_char_idx(), 5);
        assert!(state.selection().is_none());
    }

    #[test]
    fn insert_mode_toggle() {
        let mut state = InputState::default();

        assert!(!state.insert_mode);

        state.handle_message(Message::ToggleInsertMode);

        assert!(state.insert_mode);
    }
}
