use crokey::key;
use edtui::actions::DeleteLine;
use edtui::events::{KeyCombinationHandler, KeyCombinationRegister};
use edtui::{
    actions::{MoveWordBackward, MoveWordForward, SwitchMode},
    EditorEventHandler, EditorMode, EditorState, EditorView, Lines,
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    prelude::*,
    widgets::Widget,
    DefaultTerminal,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| frame.render_widget(&mut app, frame.area()))?;
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                break;
            }
            app.event_handler.on_key_event(&key.into(), &mut app.state);
        }
    }
    Ok(())
}

struct App {
    state: EditorState,
    event_handler: EditorEventHandler,
}

impl App {
    fn new() -> Self {
        let mut key_handler = KeyCombinationHandler::vim_mode();

        key_handler.insert(
            KeyCombinationRegister::n(vec![key!(ctrl-x)]),
            SwitchMode(EditorMode::Insert),
        );

        key_handler.insert(
            KeyCombinationRegister::i(vec![key!(ctrl-q)]),
            SwitchMode(EditorMode::Normal),
        );

        key_handler.insert(
            KeyCombinationRegister::n(vec![key!(ctrl-left)]),
            MoveWordBackward(1),
        );

        key_handler.insert(
            KeyCombinationRegister::n(vec![key!(ctrl-right)]),
            MoveWordForward(1),
        );

        key_handler.insert(
            KeyCombinationRegister::n(vec![key!(ctrl-alt-d)]),
            DeleteLine(1),
        );

        Self {
            state: EditorState::new(Lines::from(
                "Custom Keybindings Example

This example shows how to customize keybindings:
- Ctrl+x enters insert mode (instead of 'i')
- Ctrl+q exits insert mode (instead of Esc)

All other Vim keybindings remain active.

Try it out!
",
            )),
            event_handler: EditorEventHandler::new(key_handler),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        EditorView::new(&mut self.state)
            .wrap(true)
            .render(area, buf)
    }
}
