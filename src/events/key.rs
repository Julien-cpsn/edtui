use crate::actions::cpaste::PasteOverSelection;
use crate::actions::delete::{DeleteCharForward, DeleteToEndOfLine, DeleteToFirstCharOfLine};
use crate::actions::motion::{MoveHalfPageDown, MoveToFirstRow, MoveToLastRow};
use crate::actions::search::StartSearch;
#[cfg(feature = "system-editor")]
use crate::actions::OpenSystemEditor;
use crate::actions::{
    Action, AppendCharToSearch, AppendNewline, ChangeInnerBetween, ChangeInnerWord,
    ChangeSelection, Composed, CopyLine, CopySelection, DeleteChar, DeleteLine, DeleteSelection,
    Execute, FindFirst, FindNext, FindPrevious, InsertChar, InsertNewline, JoinLineWithLineBelow,
    LineBreak, MoveBackward, MoveDown, MoveForward, MoveHalfPageUp, MoveToEndOfLine, MoveToFirst,
    MoveToMatchinBracket, MoveToStartOfLine, MoveUp, MoveWordBackward, MoveWordForward,
    MoveWordForwardToEndOfWord, Paste, Redo, RemoveChar, RemoveCharFromSearch, SelectCurrentSearch,
    SelectInnerBetween, SelectInnerWord, SelectLine, StopSearch, SwitchMode, Undo,
};
use crate::{EditorMode, EditorState};
use crokey::{key, KeyCombination};
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct KeyCombinationHandler {
    lookup: Vec<KeyCombination>,
    register: HashMap<KeyCombinationRegister, Action>,
    capture_on_insert: bool,
}

impl Default for KeyCombinationHandler {
    fn default() -> Self {
        Self::vim_mode()
    }
}

impl KeyCombinationHandler {
    /// Creates a new `KeyCombinationHandler`.
    #[must_use]
    pub fn new(register: HashMap<KeyCombinationRegister, Action>, capture_on_insert: bool) -> Self {
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert,
        }
    }

    /// Creates a new `KeyCombinationHandler` with vim keybindings.
    #[must_use]
    pub fn vim_mode() -> Self {
        let register: HashMap<KeyCombinationRegister, Action> = vim_keybindings();
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert: false,
        }
    }

    // Creates a new `KeyCombinationHandler` with emacs keybindings.
    #[must_use]
    pub fn emacs_mode() -> Self {
        let register: HashMap<KeyCombinationRegister, Action> = emacs_keybindings();
        Self {
            lookup: Vec::new(),
            register,
            capture_on_insert: true,
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T>(&mut self, key: KeyCombinationRegister, action: T)
    where
        T: Into<Action>,
    {
        self.register.insert(key, action.into());
    }

    /// Extents the register with the contents of an iterator
    pub fn extend<T, U>(&mut self, iter: T)
    where
        U: Into<Action>,
        T: IntoIterator<Item = (KeyCombinationRegister, U)>,
    {
        self.register
            .extend(iter.into_iter().map(|(k, v)| (k, v.into())));
    }

    /// Remove a callback from the registry
    pub fn remove(&mut self, key: &KeyCombinationRegister) {
        self.register.remove(key);
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    fn get(&mut self, c: &KeyCombination, mode: EditorMode) -> Option<Action> {
        self.lookup.push(*c);
        let key = KeyCombinationRegister::new(self.lookup.clone(), mode);

        let matching_keys = self
            .register
            .iter()
            .find(|(k, _)| k.mode == key.mode && k.keys.starts_with(&key.keys));

        if let Some((_, action)) = matching_keys {
            self.lookup.clear();

            Some(action.clone())
        } else {
            self.lookup.clear();
            None
        }
    }
}

#[allow(clippy::too_many_lines)]
fn vim_keybindings() -> HashMap<KeyCombinationRegister, Action> {
    #[allow(unused_mut)]
    let mut map = HashMap::from([
        // Go into normal mode
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )]),
            SwitchMode(EditorMode::Normal).into(),
        ),
        // Go into insert mode
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('i'),
                KeyModifiers::NONE,
            )]),
            SwitchMode(EditorMode::Insert).into(),
        ),
        // Go into visual mode
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('v'),
                KeyModifiers::NONE,
            )]),
            SwitchMode(EditorMode::Visual).into(),
        ),
        // Goes into search mode and starts of a new search.
        (
            KeyCombinationRegister::n(vec![key!('/')]),
            Composed::new(StartSearch)
                .chain(SwitchMode(EditorMode::Search))
                .into(),
        ),
        // Trigger initial search
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )]),
            Composed::new(FindFirst)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
        // Find next
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('n'),
                KeyModifiers::NONE,
            )]),
            FindNext.into(),
        ),
        // Find previous
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('N'),
                KeyModifiers::NONE,
            )]),
            FindPrevious.into(),
        ),
        // Clear search
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )]),
            Composed::new(StopSearch)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
        // Delete last character from search
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            RemoveCharFromSearch.into(),
        ),
        // Go into insert mode and move one char forward
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(MoveForward(1))
                .into(),
        ),
        // Move cursor forward
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('l'),
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('l'),
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Right,
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Right,
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Right,
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        // Move cursor backward
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('h'),
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('h'),
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Left,
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Left,
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Left,
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        // Move cursor up
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Up,
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Up,
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Up,
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        // Move cursor down
        (
            KeyCombinationRegister::n(vec![key!('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![key!('j')]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Down,
                KeyModifiers::NONE,
            )]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Down,
                KeyModifiers::NONE,
            )]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Down,
                KeyModifiers::NONE,
            )]),
            MoveDown(1).into(),
        ),
        // Move one word forward/backward
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('w'),
                KeyModifiers::NONE,
            )]),
            MoveWordForward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('w'),
                KeyModifiers::NONE,
            )]),
            MoveWordForward(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )]),
            MoveWordForwardToEndOfWord(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
            )]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
            )]),
            MoveWordBackward(1).into(),
        ),
        // Move cursor to start/first/last position
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('0'),
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::n(vec![key!('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyCombinationRegister::n(vec![key!('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('0'),
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::v(vec![key!('_')]),
            MoveToFirst().into(),
        ),
        (
            KeyCombinationRegister::v(vec![key!('$')]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageUp().into(),
        ),
        // `Home` and `End` go to first/last position in a line
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Home,
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Home,
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Home,
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::End,
                KeyModifiers::NONE,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::End,
                KeyModifiers::NONE,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::End,
                KeyModifiers::NONE,
            )]),
            MoveToEndOfLine().into(),
        ),
        // `Ctrl+u` deletes from cursor to first non-whitespace character in insert mode
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            DeleteToFirstCharOfLine.into(),
        ),
        // Move cursor to start/first/last position and enter insert mode
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('I'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(MoveToFirst())
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('A'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(MoveToEndOfLine())
                .chain(MoveForward(1))
                .into(),
        ),
        // Move cursor to start/last row in the buffer
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('g'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('g'), KeyModifiers::NONE),
            ]),
            MoveToFirstRow().into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('g'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('g'), KeyModifiers::NONE),
            ]),
            MoveToFirstRow().into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('G'),
                KeyModifiers::NONE,
            )]),
            MoveToLastRow().into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('G'),
                KeyModifiers::NONE,
            )]),
            MoveToLastRow().into(),
        ),
        // Move cursor to the next opening/closing bracket.
        (
            KeyCombinationRegister::n(vec![key!('%')]),
            MoveToMatchinBracket().into(),
        ),
        (
            KeyCombinationRegister::v(vec![key!('%')]),
            MoveToMatchinBracket().into(),
        ),
        // Append/insert new line and switch into insert mode
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('o'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(AppendNewline(1))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('O'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(InsertNewline(1))
                .into(),
        ),
        // Insert a line break
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )]),
            LineBreak(1).into(),
        ),
        // Remove the current character
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('x'),
                KeyModifiers::NONE,
            )]),
            RemoveChar(1).into(),
        ),
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Delete,
                KeyModifiers::NONE,
            )]),
            RemoveChar(1).into(),
        ),
        // Delete the previous character
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            DeleteChar(1).into(),
        ),
        // Delete the next character
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Delete,
                KeyModifiers::NONE,
            )]),
            DeleteCharForward(1).into(),
        ),
        // Delete the current line
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('d'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('d'), KeyModifiers::NONE),
            ]),
            DeleteLine(1).into(),
        ),
        // Delete from the cursor to the end of the line
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('D'),
                KeyModifiers::NONE,
            )]),
            DeleteToEndOfLine.into(),
        ),
        // Delete the current selection
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
            )]),
            Composed::new(DeleteSelection)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
        // Join the current line with the line below
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('J'),
                KeyModifiers::NONE,
            )]),
            JoinLineWithLineBelow.into(),
        ),
        // Select inner word between delimiters
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('w'), KeyModifiers::NONE),
            ]),
            SelectInnerWord.into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('"'),
            ]),
            SelectInnerBetween::new('"', '"').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('\''),
            ]),
            SelectInnerBetween::new('\'', '\'').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('('),
            ]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!(')'),
            ]),
            SelectInnerBetween::new('(', ')').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('{'),
            ]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('}'),
            ]),
            SelectInnerBetween::new('{', '}').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('['),
            ]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        (
            KeyCombinationRegister::v(vec![
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!(']'),
            ]),
            SelectInnerBetween::new('[', ']').into(),
        ),
        // Change inner word between delimiters
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('w'), KeyModifiers::NONE),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerWord)
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('"'),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('"', '"'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('\''),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('\'', '\''))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('('),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!(')'),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('(', ')'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('{'),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('}'),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('{', '}'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!('['),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('i'), KeyModifiers::NONE),
                key!(']'),
            ]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeInnerBetween::new('[', ']'))
                .into(),
        ),
        // Change selection
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('c'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Insert))
                .chain(ChangeSelection)
                .into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('x'),
                KeyModifiers::NONE,
            )]),
            Composed::new(ChangeSelection)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
        // Select  the line
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('V'),
                KeyModifiers::NONE,
            )]),
            SelectLine.into(),
        ),
        // Undo
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            Undo.into(),
        ),
        // Redo
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('r'),
                KeyModifiers::NONE,
            )]),
            Redo.into(),
        ),
        // Copy
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            )]),
            Composed::new(CopySelection)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
        (
            KeyCombinationRegister::n(vec![
                KeyCombination::one_key(KeyCode::Char('y'), KeyModifiers::NONE),
                KeyCombination::one_key(KeyCode::Char('y'), KeyModifiers::NONE),
            ]),
            CopyLine.into(),
        ),
        // Paste
        (
            KeyCombinationRegister::n(vec![KeyCombination::one_key(
                KeyCode::Char('p'),
                KeyModifiers::NONE,
            )]),
            Paste.into(),
        ),
        (
            KeyCombinationRegister::v(vec![KeyCombination::one_key(
                KeyCode::Char('p'),
                KeyModifiers::NONE,
            )]),
            Composed::new(PasteOverSelection)
                .chain(SwitchMode(EditorMode::Normal))
                .into(),
        ),
    ]);

    // Open system editor (Ctrl+e in normal mode)
    #[cfg(feature = "system-editor")]
    map.insert(
        KeyCombinationRegister::n(vec![KeyCombination::one_key(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )]),
        OpenSystemEditor.into(),
    );

    map
}

#[allow(clippy::too_many_lines)]
fn emacs_keybindings() -> HashMap<KeyCombinationRegister, Action> {
    HashMap::from([
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('s'),
                KeyModifiers::NONE,
            )]),
            Composed::new(StartSearch)
                .chain(SwitchMode(EditorMode::Search))
                .into(),
        ),
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Char('s'),
                KeyModifiers::NONE,
            )]),
            FindNext.into(),
        ),
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Char('r'),
                KeyModifiers::NONE,
            )]),
            FindPrevious.into(),
        ),
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )]),
            Composed::new(SelectCurrentSearch)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Char('g'),
                KeyModifiers::NONE,
            )]),
            Composed::new(StopSearch)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyCombinationRegister::s(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            RemoveCharFromSearch.into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('f'),
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Right,
                KeyModifiers::NONE,
            )]),
            MoveForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Left,
                KeyModifiers::NONE,
            )]),
            MoveBackward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('p'),
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Up,
                KeyModifiers::NONE,
            )]),
            MoveUp(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('n'),
                KeyModifiers::NONE,
            )]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Down,
                KeyModifiers::NONE,
            )]),
            MoveDown(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('f'),
                KeyModifiers::NONE,
            )]),
            MoveWordForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
            )]),
            MoveWordBackward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('v'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageDown().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('v'),
                KeyModifiers::NONE,
            )]),
            MoveHalfPageUp().into(),
        ),
        (
            KeyCombinationRegister::i(vec![key!('<')]),
            MoveToFirstRow().into(),
        ),
        (
            KeyCombinationRegister::i(vec![key!('>')]),
            MoveToLastRow().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Home,
                KeyModifiers::NONE,
            )]),
            MoveToStartOfLine().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::End,
                KeyModifiers::NONE,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )]),
            MoveToEndOfLine().into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            DeleteToFirstCharOfLine.into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
            )]),
            DeleteToEndOfLine.into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('o'),
                KeyModifiers::NONE,
            )]),
            Composed::new(LineBreak(1))
                .chain(MoveUp(1))
                .chain(MoveToEndOfLine())
                .into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )]),
            LineBreak(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('j'),
                KeyModifiers::NONE,
            )]),
            LineBreak(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            DeleteChar(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('h'),
                KeyModifiers::NONE,
            )]),
            DeleteChar(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
            )]),
            DeleteCharForward(1).into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Visual))
                .chain(MoveWordForwardToEndOfWord(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            )]),
            Composed::new(SwitchMode(EditorMode::Visual))
                .chain(MoveWordBackward(1))
                .chain(DeleteSelection)
                .chain(SwitchMode(EditorMode::Insert))
                .into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('u'),
                KeyModifiers::NONE,
            )]),
            Undo.into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('r'),
                KeyModifiers::NONE,
            )]),
            Redo.into(),
        ),
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            )]),
            Paste.into(),
        ),
        #[cfg(feature = "system-editor")]
        (
            KeyCombinationRegister::i(vec![KeyCombination::one_key(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )]),
            OpenSystemEditor.into(),
        ),
    ])
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyCombinationRegister {
    keys: Vec<KeyCombination>,
    mode: EditorMode,
}

type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
struct RegisterVal(pub fn(&mut EditorState));

impl KeyCombinationRegister {
    pub fn new(key: Vec<KeyCombination>, mode: EditorMode) -> Self {
        Self { keys: key, mode }
    }

    pub fn n(key: Vec<KeyCombination>) -> Self {
        Self::new(key, EditorMode::Normal)
    }

    pub fn v(key: Vec<KeyCombination>) -> Self {
        Self::new(key, EditorMode::Visual)
    }

    pub fn i(key: Vec<KeyCombination>) -> Self {
        Self::new(key, EditorMode::Insert)
    }

    pub fn s(key: Vec<KeyCombination>) -> Self {
        Self::new(key, EditorMode::Search)
    }
}

impl KeyCombinationHandler {
    pub(crate) fn on_event(&mut self, key: &KeyCombination, state: &mut EditorState) {
        let mode = state.mode;

        let key_code = key.codes.first();

        match key_code {
            // Always insert characters in insert mode
            KeyCode::Char(c) if mode == EditorMode::Insert => {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar(*c).execute(state)
            }
            KeyCode::Tab if mode == EditorMode::Insert => {
                if self.capture_on_insert {
                    state.capture();
                }
                InsertChar('\t').execute(state)
            }
            // Always add characters to search in search mode
            KeyCode::Char(c) if mode == EditorMode::Search => AppendCharToSearch(*c).execute(state),
            // Else lookup an action from the register
            _ => {
                if let Some(mut action) = self.get(key, mode) {
                    action.execute(state);
                }
            }
        }
    }
}
