use std::{
    cmp::{max, min},
    ops::{Deref, Range},
};

use crate::Message;

/// Stored state of the input widget. Used for the cursor position, text selection and windowing/scrolling
#[derive(Debug)]
pub struct InputState {
    value: String,
    cursor_char_idx: usize,
    in_focus: bool,
    insert_mode: bool,
    selection_start_char_idx: Option<usize>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_char_idx: 0,
            in_focus: false,
            insert_mode: false,
            selection_start_char_idx: None,
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
                    }
                    None => {
                        if self.cursor_char_idx == self.value.chars().count() {
                            // Do nothing because we are not currently on a character
                        } else {
                            // Remove the character from the string
                            let idx = char_idx_to_byte_idx(&self.value, self.cursor_char_idx);
                            let _ = self.value.remove(idx);
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
                }
            }
            Message::MoveRight => {
                // End selection
                self.selection_start_char_idx = None;

                if self.cursor_char_idx == self.value.chars().count() {
                    // We are already 1 step ahead of the value, so we cannot move anymore
                } else {
                    self.cursor_char_idx += 1;
                }
            }
            Message::JumpToEnd => self.cursor_char_idx = self.value.chars().count(),
            Message::JumpToStart => self.cursor_char_idx = 0,
            Message::Char(c) => {
                match self.selection() {
                    Some(selection) => {
                        // Replace the entire selection with the input
                        self.value
                            .replace_range(selection.byte_range, c.to_string().as_str());
                        self.cursor_char_idx = selection.char_range.start;
                        self.selection_start_char_idx = None;
                    }
                    None => {
                        if self.cursor_char_idx == self.value.chars().count() {
                            // We are outside the string, so we push onto it
                            self.value.push(c);
                        } else {
                            if self.insert_mode {
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
                        }
                        self.cursor_char_idx += 1;
                    }
                }
            }
            Message::Paste(str) => match self.selection() {
                Some(selection) => {
                    self.value.replace_range(selection.byte_range, &str);
                    self.cursor_char_idx += str.chars().count();
                    self.selection_start_char_idx = None;
                }
                None => {
                    if self.cursor_char_idx == self.value.chars().count() {
                        self.value.push_str(str.as_str());
                    } else {
                        self.value
                            .insert_str(self.cursor_char_idx() + 1, str.as_str());
                    }
                    self.cursor_char_idx += str.chars().count();
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
            }
            Message::Copy => match self.selection() {
                Some(selection) => {
                    // Copy the selection
                    clipboard_win::set_clipboard_string(&selection).unwrap();
                }
                None => {
                    // No selection, so we copy the entire value
                    clipboard_win::set_clipboard_string(&self.value).unwrap()
                }
            },
            Message::Cut => match self.selection() {
                Some(selection) => {
                    // Cut the selection and set cursor to the start of the selecion
                    clipboard_win::set_clipboard_string(&selection).unwrap();
                    self.cursor_char_idx = selection.char_range.start;
                    self.selection_start_char_idx = None;
                    let mut taken_iter = (0..self.value.chars().count())
                        .map(|char_idx| selection.char_range.contains(&char_idx));
                    self.value.retain(|_| !taken_iter.next().unwrap());
                }
                None => {
                    // Copy the entire value and then clear it
                    clipboard_win::set_clipboard_string(&self.value).unwrap();
                    self.value.clear();
                    self.cursor_char_idx = 0;
                    self.selection_start_char_idx = None;
                }
            },
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
                let max_char_idx = max(start_char_idx, self.cursor_char_idx);
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

    pub(crate) fn selection_char_range(&self) -> Option<Range<usize>> {
        self.selection().map(|x| x.char_range)
    }

    #[allow(unused)]
    pub(crate) fn selection_byte_range(&self) -> Option<Range<usize>> {
        self.selection().map(|x| x.byte_range)
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
    char_range: Range<usize>,
    byte_range: Range<usize>,
    text: String,
}

impl Deref for Selection {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}
