use std::io::stdout;

use crossterm::{
    event::{self, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui_input::{Input, InputState};

fn main() -> Result<(), std::io::Error> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = InputState::default();

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let input = Input::new();
            frame.render_stateful_widget(input, area, &mut state);
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.code == KeyCode::Esc {
                    break;
                } else {
                    state.handle_message(event.into());
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
