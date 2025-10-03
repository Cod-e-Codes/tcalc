use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

mod calculator;
mod ui;

use calculator::CalculatorModule;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Normal, // Button navigation mode
    Typing, // Direct typing mode
}

pub struct App {
    pub state: AppState,
    pub calculator_module: CalculatorModule,
    pub button_position: Option<(usize, usize)>, // (row, col)
    pub show_history: bool,
    pub history_selected: usize,
    pub scroll_offset: usize,
    pub status_message: String,
    pub mouse_position: Option<(u16, u16)>, // (x, y) for hover tracking
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Normal,
            calculator_module: CalculatorModule::new(),
            button_position: Some((0, 0)),
            show_history: false,
            history_selected: 0,
            scroll_offset: 0,
            status_message: "Calculator ready. Press ` for typing mode, ? for help".to_string(),
            mouse_position: None,
        }
    }

    pub fn get_calculator_buttons(&self) -> Vec<Vec<(&'static str, &'static str)>> {
        match self.calculator_module.mode {
            calculator::CalculatorMode::Basic => vec![
                vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                vec![("7", "7"), ("8", "8"), ("9", "9"), ("×", "*")],
                vec![("4", "4"), ("5", "5"), ("6", "6"), ("−", "-")],
                vec![("1", "1"), ("2", "2"), ("3", "3"), ("+", "+")],
                vec![("(", "("), ("0", "0"), (")", ")"), (".", ".")],
                vec![("^", "^"), ("%", "%"), ("=", "enter"), ("Copy", "y")],
            ],
            calculator::CalculatorMode::Scientific => vec![
                vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                vec![("sin", "s"), ("cos", "c"), ("tan", "t"), ("×", "*")],
                vec![("√", "q"), ("log", "l"), ("ln", "n"), ("−", "-")],
                vec![("7", "7"), ("8", "8"), ("9", "9"), ("+", "+")],
                vec![("4", "4"), ("5", "5"), ("6", "6"), ("^", "^")],
                vec![("1", "1"), ("2", "2"), ("3", "3"), ("%", "%")],
                vec![("exp", "e"), ("0", "0"), (".", "."), ("=", "enter")],
                vec![("abs", "a"), ("1/x", "i"), ("x²", "x"), ("Copy", "y")],
            ],
        }
    }

    pub fn press_button(&mut self) {
        if let Some((row, col)) = self.button_position {
            let buttons = self.get_calculator_buttons();
            let actual_row = self.scroll_offset + row;
            if actual_row < buttons.len()
                && let Some((_, key)) = buttons[actual_row].get(col)
            {
                    match *key {
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => self
                            .calculator_module
                            .append_digit(key.chars().next().unwrap()),
                        "+" => self.calculator_module.append_operator("+"),
                        "-" => self.calculator_module.append_operator("-"),
                        "*" => self.calculator_module.append_operator("*"),
                        "/" => self.calculator_module.append_operator("/"),
                        "^" => self.calculator_module.append_operator("^"),
                        "%" => self.calculator_module.append_operator("%"),
                        "(" => {
                            self.calculator_module.current_expression.push('(');
                            self.calculator_module.update_result();
                        }
                        ")" => {
                            self.calculator_module.current_expression.push(')');
                            self.calculator_module.update_result();
                        }
                        "." => self.calculator_module.append_decimal(),
                        "enter" => self.calculator_module.calculate(),
                        "bksp" => self.calculator_module.backspace(),
                        "c" => self.calculator_module.clear(),
                        "C" => {
                            self.calculator_module.clear_all();
                            self.history_selected = 0;
                        }
                        "y" => {
                            self.status_message =
                                format!("Result: {}", self.calculator_module.current_result);
                        }
                        // Scientific functions
                        "s" => self.calculator_module.apply_function("sin"),
                        "t" => self.calculator_module.apply_function("tan"),
                        "q" => self.calculator_module.apply_function("sqrt"),
                        "l" => self.calculator_module.apply_function("log"),
                        "n" => self.calculator_module.apply_function("ln"),
                        "e" => self.calculator_module.apply_function("exp"),
                        "a" => self.calculator_module.apply_function("abs"),
                        "i" => self.calculator_module.apply_function("1/x"),
                        "x" => self.calculator_module.apply_function("x^2"),
                        _ => {}
                    }
            }
        }
    }

    pub fn button_up(&mut self) {
        if let Some((row, col)) = self.button_position {
            if row > 0 {
                self.button_position = Some((row - 1, col));
            } else if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
        }
    }

    pub fn button_down(&mut self) {
        if let Some((row, col)) = self.button_position {
            let buttons = self.get_calculator_buttons();
            if (self.scroll_offset + row + 1) < buttons.len() {
                if row < 5 {
                    self.button_position = Some((row + 1, col));
                } else {
                    self.scroll_offset += 1;
                }
            }
        }
    }

    pub fn button_left(&mut self) {
        if let Some((row, col)) = self.button_position
            && col > 0
        {
            self.button_position = Some((row, col - 1));
        }
    }

    pub fn button_right(&mut self) {
        if let Some((row, col)) = self.button_position {
            let buttons = self.get_calculator_buttons();
            if col < buttons[row].len() - 1 {
                self.button_position = Some((row, col + 1));
            }
        }
    }

    pub fn toggle_mode(&mut self) {
        self.calculator_module.toggle_mode();
        self.button_position = Some((0, 0));
        self.scroll_offset = 0;
    }

    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
        if self.show_history {
            self.status_message =
                "History view - h to toggle back, ↑↓ navigate, r to recall".to_string();
        } else {
            self.status_message = "Calculator mode".to_string();
        }
    }

    pub fn history_next(&mut self) {
        if !self.calculator_module.history.is_empty() {
            self.history_selected =
                (self.history_selected + 1) % self.calculator_module.history.len();
        }
    }

    pub fn history_prev(&mut self) {
        if !self.calculator_module.history.is_empty() {
            if self.history_selected == 0 {
                self.history_selected = self.calculator_module.history.len() - 1;
            } else {
                self.history_selected -= 1;
            }
        }
    }

    pub fn recall_from_history(&mut self) {
        self.calculator_module
            .recall_from_history(self.history_selected);
    }

    pub fn mouse_to_button_coords(&self, x: u16, y: u16, terminal_width: u16) -> Option<(usize, usize)> {
        // Only work in normal mode and when not showing history
        if self.state != AppState::Normal || self.show_history {
            return None;
        }

        // Calculate the button area bounds more accurately
        // Title: 3 lines, Display: 6 lines, so buttons start at y = 9
        let button_start_y = 9;
        let button_height = 3; // Each button row is 3 lines high
        
        if y < button_start_y {
            return None; // Clicked above button area
        }

        let button_row = (y - button_start_y) / button_height;
        
        // Calculate button column based on terminal width and number of buttons per row
        let buttons = self.get_calculator_buttons();
        let actual_row = self.scroll_offset + button_row as usize;
        
        if actual_row >= buttons.len() {
            return None; // Clicked below button area
        }
        
        let buttons_in_row = buttons[actual_row].len();
        let button_width = terminal_width / buttons_in_row as u16;
        let button_col = x / button_width;
        
        if button_col < buttons_in_row as u16 {
            Some((button_row as usize, button_col as usize))
        } else {
            None
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn handle_mouse_click(app: &mut App, x: u16, y: u16, terminal_width: u16) {
    if let Some((row, col)) = app.mouse_to_button_coords(x, y, terminal_width) {
        app.button_position = Some((row, col));
        app.press_button();
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app, f.area()))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) => {
                    if kind != KeyEventKind::Press {
                        continue;
                    }

            match app.state {
                AppState::Normal => match code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('?') => {
                        app.status_message = "Help: ` typing mode | m toggle mode | h history | ↑↓←→ navigate | Enter/Space/Mouse click press button | q quit".to_string();
                    }
                    KeyCode::Char('`') => {
                        app.state = AppState::Typing;
                        app.status_message =
                            "Typing mode - type expressions, ` to exit".to_string();
                    }
                    KeyCode::Up => app.button_up(),
                    KeyCode::Down => app.button_down(),
                    KeyCode::Left => app.button_left(),
                    KeyCode::Right => app.button_right(),
                    KeyCode::Enter | KeyCode::Char(' ') => app.press_button(),
                    KeyCode::Char('m') => app.toggle_mode(),
                    KeyCode::Char('h') => app.toggle_history(),
                    KeyCode::Char('r') => app.recall_from_history(),
                    _ => {}
                },
                AppState::Typing => match code {
                    KeyCode::Char('`') | KeyCode::Esc => {
                        app.state = AppState::Normal;
                        app.status_message = "Button navigation mode".to_string();
                    }
                    KeyCode::Up => app.history_prev(),
                    KeyCode::Down => app.history_next(),
                    KeyCode::Char(c @ '0'..='9') => app.calculator_module.append_digit(c),
                    KeyCode::Char('.') => app.calculator_module.append_decimal(),
                    KeyCode::Char('+') => app.calculator_module.append_operator("+"),
                    KeyCode::Char('-') => app.calculator_module.append_operator("-"),
                    KeyCode::Char('*') => app.calculator_module.append_operator("*"),
                    KeyCode::Char('/') => app.calculator_module.append_operator("/"),
                    KeyCode::Char('^') => app.calculator_module.append_operator("^"),
                    KeyCode::Char('%') => app.calculator_module.append_operator("%"),
                    KeyCode::Char('(') => {
                        app.calculator_module.current_expression.push('(');
                        app.calculator_module.update_result();
                    }
                    KeyCode::Char(')') => {
                        app.calculator_module.current_expression.push(')');
                        app.calculator_module.update_result();
                    }
                    KeyCode::Enter => app.calculator_module.calculate(),
                    KeyCode::Backspace => app.calculator_module.backspace(),
                    KeyCode::Char('c') if !modifiers.contains(KeyModifiers::SHIFT) => {
                        app.calculator_module.clear();
                    }
                    KeyCode::Char('C') => {
                        app.calculator_module.clear_all();
                        app.history_selected = 0;
                    }
                    KeyCode::Char('m') => app.toggle_mode(),
                    KeyCode::Char('h') => app.toggle_history(),
                    KeyCode::Char('r') => app.recall_from_history(),
                    // Scientific functions (only in scientific mode)
                    KeyCode::Char('s')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("sin");
                    }
                    KeyCode::Char('t')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("tan");
                    }
                    KeyCode::Char('q')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("sqrt");
                    }
                    KeyCode::Char('l')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("log");
                    }
                    KeyCode::Char('n')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("ln");
                    }
                    KeyCode::Char('e')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("exp");
                    }
                    KeyCode::Char('a')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("abs");
                    }
                    KeyCode::Char('i')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("1/x");
                    }
                    KeyCode::Char('x')
                        if app.calculator_module.mode == calculator::CalculatorMode::Scientific =>
                    {
                        app.calculator_module.apply_function("x^2");
                    }
                    _ => {}
                },
            }
                }
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            let terminal_size = terminal.size()?;
                            handle_mouse_click(app, mouse_event.column, mouse_event.row, terminal_size.width);
                        }
                        crossterm::event::MouseEventKind::Moved => {
                            // Track mouse position for hover effects
                            app.mouse_position = Some((mouse_event.column, mouse_event.row));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
