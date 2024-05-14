use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Messages passed to the InputState
#[derive(Debug)]
pub enum Message {
    /// Empty message, no state change happens. Used to avoid the use of [`Option<Message>`].
    Empty,
    /// Gain focus on the input
    Focus,
    /// Remove focus from the input
    RemoveFocus,
    /// Delete the character under the cursor
    DeleteOnCursor,
    /// Delete the character before the cursor
    DeleteBeforeCursor,
    /// Move the cursor to the left
    MoveLeft,
    /// Move the cursor to the left and start/continou text selection
    MoveLeftWithSelection,
    /// Move the cursor to the right
    MoveRight,
    /// Move the cursor to the right and start/continue the text selection
    MoveRightWithSelection,
    /// Jump the cursor to the end
    JumpToEnd,
    /// Jump the cursor to the end and seect everything in between the end and start position
    JumpToEndWithSelection,
    /// Jump the cursor to the start
    JumpToStart,
    /// Jump the cursor to the start and select everything in between the start and end position
    JumpToStartWithSelection,
    /// Character input
    Char(char),
    /// Insert a string at the current cursor position. If we have a selection, the selection will get replaced
    Paste(String),
    /// Toggle the insert mode
    ToggleInsertMode,
    /// Copy selected text or if there is no selection, the entire input value and add it to the clipboard
    #[cfg(target_os = "windows")]
    Copy,
    /// Cut selected text or if there is no selection the entire input and add it to the clipboard
    #[cfg(target_os = "windows")]
    Cut,
    //TODO: SelectAll
    //TODO: SelectWord
    //TODO: JumpToEndOfWord
    //TODO: JumpToStartOfWord
}

impl From<Event> for Message {
    fn from(value: Event) -> Self {
        match value {
            Event::FocusGained => Message::Focus,
            Event::FocusLost => Message::RemoveFocus,
            Event::Key(key) => key.into(),
            Event::Mouse(_) => Message::Empty,
            Event::Paste(str) => Message::Paste(str),
            Event::Resize(_, _) => Message::Empty,
        }
    }
}

impl From<KeyEvent> for Message {
    fn from(value: KeyEvent) -> Self {
        if value.kind == KeyEventKind::Release {
            Message::Empty
        } else {
            match value.code {
                KeyCode::Backspace => Message::DeleteBeforeCursor,
                KeyCode::Enter => Message::RemoveFocus,
                KeyCode::Left => {
                    if value.modifiers == KeyModifiers::SHIFT {
                        Message::MoveLeftWithSelection
                    } else {
                        Message::MoveLeft
                    }
                }
                KeyCode::Right => {
                    if value.modifiers == KeyModifiers::SHIFT {
                        Message::MoveRightWithSelection
                    } else {
                        Message::MoveRight
                    }
                }
                KeyCode::Up => Message::Empty,
                KeyCode::Down => Message::Empty,
                KeyCode::Home => {
                    if value.modifiers == KeyModifiers::SHIFT {
                        Message::JumpToStartWithSelection
                    } else {
                        Message::JumpToStart
                    }
                }
                KeyCode::End => {
                    if value.modifiers == KeyModifiers::SHIFT {
                        Message::JumpToEndWithSelection
                    } else {
                        Message::JumpToEnd
                    }
                }
                KeyCode::PageUp => Message::Empty,
                KeyCode::PageDown => Message::Empty,
                KeyCode::Tab => Message::Char('\t'),
                KeyCode::BackTab => Message::Empty,
                KeyCode::Delete => Message::DeleteOnCursor,
                KeyCode::Insert => Message::ToggleInsertMode,
                KeyCode::F(_) => Message::Empty,
                KeyCode::Char(c) => match c {
                    'c' => {
                        if value.modifiers == KeyModifiers::CONTROL {
                            if cfg!(target_os = "windows") {
                                Message::Copy
                            } else {
                                Message::Empty
                            }
                        } else {
                            Message::Char('c')
                        }
                    }
                    'x' => {
                        if value.modifiers == KeyModifiers::CONTROL {
                            if cfg!(target_os = "windows") {
                                Message::Cut
                            } else {
                                Message::Empty
                            }
                        } else {
                            Message::Char('x')
                        }
                    }
                    'v' => {
                        if value.modifiers == KeyModifiers::CONTROL {
                            match clipboard_win::get_clipboard_string() {
                                Ok(str) => Message::Paste(str),
                                Err(_) => Message::Empty,
                            }
                        } else {
                            Message::Char('v')
                        }
                    }
                    c => Message::Char(c),
                },
                KeyCode::Null => Message::Empty,
                KeyCode::Esc => Message::RemoveFocus,
                KeyCode::CapsLock => Message::Empty,
                KeyCode::ScrollLock => Message::Empty,
                KeyCode::NumLock => Message::Empty,
                KeyCode::PrintScreen => Message::Empty,
                KeyCode::Pause => Message::Empty,
                KeyCode::Menu => Message::Empty,
                KeyCode::KeypadBegin => Message::Empty,
                KeyCode::Media(_) => Message::Empty,
                KeyCode::Modifier(_) => Message::Empty,
            }
        }
    }
}
