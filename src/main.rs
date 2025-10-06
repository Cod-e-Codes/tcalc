use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use std::io;
use std::time::{Duration, Instant};

mod calculator;
mod graph;
mod ui;

use calculator::CalculatorModule;
use graph::GraphModule;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Normal, // Button navigation mode
    Typing, // Direct typing mode
    Graph,  // Graph mode for plotting expressions
}

pub struct App {
    pub state: AppState,
    pub calculator_module: CalculatorModule,
    pub graph_module: GraphModule,
    pub button_position: Option<(usize, usize)>, // (row, col)
    pub show_history: bool,
    pub history_selected: usize,
    pub scroll_offset: usize,
    pub status_message: String,
    pub mouse_position: Option<(u16, u16)>, // (x, y) for hover tracking
    pub graph_expression: String,
    pub graph_x_min: f64,
    pub graph_x_max: f64,
    pub graph_y_min: f64,
    pub graph_y_max: f64,
    pub graph_cursor_x: f64,
    pub graph_cursor_y: f64,
    pub show_cursor_coords: bool,
    pub second_function_mode: bool, // For 2nd function key
    pub show_help: bool,
    pub last_nav_time: Option<Instant>,
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
            graph_module: GraphModule::new(),
            button_position: None, // No selection by default
            show_history: false,
            history_selected: 0,
            scroll_offset: 0,
            status_message: "Calculator ready. Press ` for typing mode, ? for help".to_string(),
            mouse_position: None,
            graph_expression: String::new(),
            graph_x_min: -10.0,
            graph_x_max: 10.0,
            graph_y_min: -10.0,
            graph_y_max: 10.0,
            graph_cursor_x: 0.0,
            graph_cursor_y: 0.0,
            show_cursor_coords: true,
            second_function_mode: false,
            show_help: false,
            last_nav_time: None,
        }
    }

    pub fn get_calculator_buttons(&self) -> Vec<Vec<(&'static str, &'static str)>> {
        if self.second_function_mode {
            // Secondary function mode - show variables and advanced functions
            match self.calculator_module.mode {
                calculator::CalculatorMode::Basic => vec![
                    vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                    vec![("x", "x"), ("y", "y"), ("z", "z"), ("×", "*")],
                    vec![("a", "a"), ("b", "b"), ("c", "c"), ("−", "-")],
                    vec![("π", "pi"), ("e", "e"), ("(", "("), (")", ")")],
                    vec![("^", "^"), ("%", "%"), ("Graph", "g"), ("2nd", "2nd")],
                ],
                calculator::CalculatorMode::Scientific => vec![
                    vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                    vec![("x", "x"), ("y", "y"), ("z", "z"), ("×", "*")],
                    vec![("a", "a"), ("b", "b"), ("c", "c"), ("−", "-")],
                    vec![("sin", "s"), ("cos", "c"), ("tan", "t"), ("+", "+")],
                    vec![("√", "q"), ("log", "l"), ("ln", "n"), ("^", "^")],
                    vec![("exp", "e"), ("0", "0"), (".", "."), ("=", "enter")],
                    vec![("abs", "a"), ("1/x", "i"), ("x²", "x"), ("%", "%")],
                    vec![("π", "pi"), ("e", "e"), ("Graph", "g"), ("2nd", "2nd")],
                ],
            }
        } else {
            // Primary function mode - show numbers and basic operations
            match self.calculator_module.mode {
                calculator::CalculatorMode::Basic => vec![
                    vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                    vec![("7", "7"), ("8", "8"), ("9", "9"), ("×", "*")],
                    vec![("4", "4"), ("5", "5"), ("6", "6"), ("−", "-")],
                    vec![("1", "1"), ("2", "2"), ("3", "3"), ("+", "+")],
                    vec![("(", "("), ("0", "0"), (")", ")"), (".", ".")],
                    vec![("^", "^"), ("%", "%"), ("=", "enter"), ("2nd", "2nd")],
                ],
                calculator::CalculatorMode::Scientific => vec![
                    vec![("C", "c"), ("CE", "C"), ("⌫", "bksp"), ("÷", "/")],
                    vec![("7", "7"), ("8", "8"), ("9", "9"), ("×", "*")],
                    vec![("4", "4"), ("5", "5"), ("6", "6"), ("−", "-")],
                    vec![("1", "1"), ("2", "2"), ("3", "3"), ("+", "+")],
                    vec![("(", "("), ("0", "0"), (")", ")"), (".", ".")],
                    vec![("^", "^"), ("%", "%"), ("=", "enter"), ("2nd", "2nd")],
                ],
            }
        }
    }

    pub fn press_button(&mut self) {
        if let Some((row, col)) = self.button_position {
            let buttons = self.get_calculator_buttons();
            let actual_row = self.scroll_offset + row;
            if actual_row < buttons.len() && buttons[actual_row].get(col).is_some() {
                // Also fetch the label to disambiguate collisions (e.g., cos vs clear, variable 'c')
                let (label, key) = buttons[actual_row][col];
                match key {
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
                    // 'c' key conflicts: could be clear current, cos function, or variable 'c'
                    "c" => {
                        if label == "cos" {
                            self.calculator_module.apply_function("cos");
                        } else if self.second_function_mode && label == "c" {
                            self.calculator_module.current_expression.push('c');
                            self.calculator_module.update_result();
                        } else {
                            self.calculator_module.clear();
                        }
                    }
                    "C" => {
                        self.calculator_module.clear_all();
                        self.history_selected = 0;
                    }
                    // Scientific functions by label disambiguation
                    "s" => self.calculator_module.apply_function("sin"),
                    "t" => self.calculator_module.apply_function("tan"),
                    "q" => self.calculator_module.apply_function("sqrt"),
                    "l" => self.calculator_module.apply_function("log"),
                    "n" => self.calculator_module.apply_function("ln"),
                    // 'a' could be abs function or variable 'a' in 2nd mode
                    "a" => {
                        if label == "abs" {
                            self.calculator_module.apply_function("abs");
                        } else if self.second_function_mode && label == "a" {
                            self.calculator_module.current_expression.push('a');
                            self.calculator_module.update_result();
                        }
                    }
                    // 'e' could be exp() function or Euler's constant
                    "e" => {
                        if label == "exp" {
                            self.calculator_module.apply_function("exp");
                        } else if label == "e" {
                            self.calculator_module
                                .current_expression
                                .push_str("2.71828");
                            self.calculator_module.update_result();
                        }
                    }
                    "i" => self.calculator_module.apply_function("1/x"),
                    "x" => {
                        // Check if this is the x² function or the x variable button
                        if label == "x²" {
                            self.calculator_module.apply_function("x^2");
                        } else if label == "x" {
                            self.calculator_module.current_expression.push('x');
                            self.calculator_module.update_result();
                        }
                    }
                    // Variables y, z, b only in 2nd function mode (a and c handled above)
                    "y" | "z" | "b" => {
                        if self.second_function_mode {
                            let ch = key.chars().next().unwrap();
                            self.calculator_module.current_expression.push(ch);
                            self.calculator_module.update_result();
                        }
                    }
                    "g" => {
                        if self.second_function_mode {
                            self.enter_graph_mode();
                        } else {
                            self.calculator_module.calculate();
                        }
                    }
                    "2nd" => self.toggle_second_function(),
                    "pi" => {
                        self.calculator_module
                            .current_expression
                            .push_str("3.14159");
                        self.calculator_module.update_result();
                    }
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
        } else {
            // First navigation - set to (0, 0)
            self.button_position = Some((0, 0));
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
        } else {
            // First navigation - set to (0, 0)
            self.button_position = Some((0, 0));
        }
    }

    fn can_navigate(&mut self) -> bool {
        let now = Instant::now();
        match self.last_nav_time {
            None => {
                self.last_nav_time = Some(now);
                true
            }
            Some(t) => {
                if now.duration_since(t) >= Duration::from_millis(120) {
                    self.last_nav_time = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn button_left(&mut self) {
        if let Some((row, col)) = self.button_position
            && col > 0
        {
            self.button_position = Some((row, col - 1));
        } else {
            // First navigation - set to (0, 0)
            self.button_position = Some((0, 0));
        }
    }

    pub fn button_right(&mut self) {
        if let Some((row, col)) = self.button_position {
            let buttons = self.get_calculator_buttons();
            if col < buttons[row].len() - 1 {
                self.button_position = Some((row, col + 1));
            }
        } else {
            // First navigation - set to (0, 0)
            self.button_position = Some((0, 0));
        }
    }

    pub fn toggle_mode(&mut self) {
        self.calculator_module.toggle_mode();
        self.button_position = None; // Clear selection when switching modes
        self.scroll_offset = 0;
    }

    pub fn toggle_second_function(&mut self) {
        self.second_function_mode = !self.second_function_mode;
        self.button_position = None; // Clear selection when switching modes
        self.scroll_offset = 0;
        if self.second_function_mode {
            self.status_message =
                "2nd function mode - Press 2nd again to return to primary functions".to_string();
        } else {
            self.status_message =
                "Primary function mode - Press 2nd for variables and advanced functions"
                    .to_string();
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.status_message = "Help - Press ? or Esc to close".to_string();
        } else {
            self.status_message =
                "Calculator ready. Press ` for typing mode, ? for help".to_string();
        }
    }

    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
        if self.show_history {
            // Select newest entry by default
            if !self.calculator_module.history.is_empty() {
                self.history_selected = self.calculator_module.history.len() - 1;
            } else {
                self.history_selected = 0;
            }
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
        // Exit history back to calculator view after recall
        self.show_history = false;
        self.status_message = "Calculator mode".to_string();
    }

    pub fn enter_graph_mode(&mut self) {
        if !self.calculator_module.current_expression.is_empty() {
            self.graph_expression = self.calculator_module.current_expression.clone();
            self.graph_module.x_min = self.graph_x_min;
            self.graph_module.x_max = self.graph_x_max;
            self.graph_module.y_min = self.graph_y_min;
            self.graph_module.y_max = self.graph_y_max;

            // Generate initial graph points
            if let Err(e) = self
                .graph_module
                .generate_points(&self.graph_expression, 100, 50)
            {
                self.status_message = format!("Error generating graph: {}", e);
                return;
            }

            self.state = AppState::Graph;
            self.status_message =
                "Graph mode - Esc to exit, arrows to pan, +/- to zoom".to_string();
        } else {
            self.status_message = "Enter an expression first, then press Graph".to_string();
        }
    }

    pub fn exit_graph_mode(&mut self) {
        self.state = AppState::Normal;
        self.status_message = "Calculator ready. Press ` for typing mode, ? for help".to_string();
    }

    pub fn pan_graph(&mut self, dx: f64, dy: f64) {
        let x_range = self.graph_x_max - self.graph_x_min;
        let y_range = self.graph_y_max - self.graph_y_min;

        self.graph_x_min += dx * x_range * 0.1;
        self.graph_x_max += dx * x_range * 0.1;
        self.graph_y_min += dy * y_range * 0.1;
        self.graph_y_max += dy * y_range * 0.1;

        // Update graph module bounds
        self.graph_module.x_min = self.graph_x_min;
        self.graph_module.x_max = self.graph_x_max;
        self.graph_module.y_min = self.graph_y_min;
        self.graph_module.y_max = self.graph_y_max;

        // Regenerate graph points
        if let Err(e) = self
            .graph_module
            .generate_points(&self.graph_expression, 100, 50)
        {
            self.status_message = format!("Error regenerating graph: {}", e);
        }
    }

    pub fn zoom_graph(&mut self, factor: f64) {
        let x_center = (self.graph_x_min + self.graph_x_max) / 2.0;
        let y_center = (self.graph_y_min + self.graph_y_max) / 2.0;
        let x_range = self.graph_x_max - self.graph_x_min;
        let y_range = self.graph_y_max - self.graph_y_min;

        let new_x_range = x_range / factor;
        let new_y_range = y_range / factor;

        self.graph_x_min = x_center - new_x_range / 2.0;
        self.graph_x_max = x_center + new_x_range / 2.0;
        self.graph_y_min = y_center - new_y_range / 2.0;
        self.graph_y_max = y_center + new_y_range / 2.0;

        // Update graph module bounds
        self.graph_module.x_min = self.graph_x_min;
        self.graph_module.x_max = self.graph_x_max;
        self.graph_module.y_min = self.graph_y_min;
        self.graph_module.y_max = self.graph_y_max;

        // Regenerate graph points
        if let Err(e) = self
            .graph_module
            .generate_points(&self.graph_expression, 100, 50)
        {
            self.status_message = format!("Error regenerating graph: {}", e);
        }
    }

    pub fn update_graph_cursor(&mut self, x: u16, y: u16, graph_area: Rect) {
        if x >= graph_area.x
            && x < graph_area.x + graph_area.width
            && y >= graph_area.y
            && y < graph_area.y + graph_area.height
        {
            let x_ratio = (x - graph_area.x) as f64 / graph_area.width as f64;
            let y_ratio = (y - graph_area.y) as f64 / graph_area.height as f64;

            self.graph_cursor_x =
                self.graph_x_min + x_ratio * (self.graph_x_max - self.graph_x_min);
            self.graph_cursor_y =
                self.graph_y_max - y_ratio * (self.graph_y_max - self.graph_y_min);
        }
    }

    pub fn mouse_to_button_coords(
        &self,
        x: u16,
        y: u16,
        terminal_width: u16,
    ) -> Option<(usize, usize)> {
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
        // Set position temporarily for button press
        app.button_position = Some((row, col));
        app.press_button();
        // Clear selection after mouse click to avoid persistent selection
        app.button_position = None;
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
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => {
                                if app.show_help {
                                    app.toggle_help();
                                } else {
                                    return Ok(());
                                }
                            }
                            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            KeyCode::Char('?') => {
                                app.toggle_help();
                            }
                            KeyCode::Char('`') => {
                                app.state = AppState::Typing;
                                app.status_message =
                                    "Typing mode - type expressions, ` to exit".to_string();
                            }
                            KeyCode::Up => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                if app.show_history {
                                    app.history_next();
                                } else {
                                    app.button_up();
                                }
                            }
                            KeyCode::Down => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                if app.show_history {
                                    app.history_prev();
                                } else {
                                    app.button_down();
                                }
                            }
                            KeyCode::Left => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                if !app.show_history {
                                    app.button_left();
                                }
                            }
                            KeyCode::Right => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                if !app.show_history {
                                    app.button_right();
                                }
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                if app.show_history {
                                    app.recall_from_history();
                                } else {
                                    app.press_button();
                                }
                            }
                            KeyCode::Char('m') => app.toggle_mode(),
                            KeyCode::Char('2') => app.toggle_second_function(),
                            KeyCode::Char('h') => app.toggle_history(),
                            KeyCode::Char('r') => {
                                if app.show_history {
                                    app.recall_from_history();
                                }
                            }
                            KeyCode::Char('g') if modifiers.contains(KeyModifiers::CONTROL) => {
                                app.enter_graph_mode()
                            }
                            _ => {}
                        },
                        AppState::Typing => match code {
                            KeyCode::Char('`') | KeyCode::Esc => {
                                app.state = AppState::Normal;
                                app.status_message = "Button navigation mode".to_string();
                            }
                            KeyCode::Up => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.history_next()
                            }
                            KeyCode::Down => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.history_prev()
                            }
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
                            KeyCode::Char('m') => app.toggle_mode(),
                            KeyCode::Char('h') => app.toggle_history(),
                            KeyCode::Char('g') if modifiers.contains(KeyModifiers::CONTROL) => {
                                app.enter_graph_mode()
                            }
                            KeyCode::Char('?') => {
                                app.toggle_help();
                            }
                            // In Typing mode, allow letters to build identifiers (functions/variables)
                            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                                app.calculator_module.current_expression.push(c);
                                app.calculator_module.update_result();
                            }
                            _ => {}
                        },
                        AppState::Graph => match code {
                            KeyCode::Esc => app.exit_graph_mode(),
                            KeyCode::Up => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.pan_graph(0.0, 1.0)
                            }
                            KeyCode::Down => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.pan_graph(0.0, -1.0)
                            }
                            KeyCode::Left => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.pan_graph(-1.0, 0.0)
                            }
                            KeyCode::Right => {
                                if !app.can_navigate() {
                                    continue;
                                }
                                app.pan_graph(1.0, 0.0)
                            }
                            KeyCode::Char('+') => app.zoom_graph(1.2),
                            KeyCode::Char('-') => app.zoom_graph(0.8),
                            KeyCode::Char('r') => {
                                // Reset view
                                app.graph_x_min = -10.0;
                                app.graph_x_max = 10.0;
                                app.graph_y_min = -10.0;
                                app.graph_y_max = 10.0;
                                app.graph_module.x_min = app.graph_x_min;
                                app.graph_module.x_max = app.graph_x_max;
                                app.graph_module.y_min = app.graph_y_min;
                                app.graph_module.y_max = app.graph_y_max;

                                // Regenerate graph points
                                if let Err(e) =
                                    app.graph_module
                                        .generate_points(&app.graph_expression, 100, 50)
                                {
                                    app.status_message = format!("Error regenerating graph: {}", e);
                                }
                            }
                            KeyCode::Char('c') => {
                                app.show_cursor_coords = !app.show_cursor_coords;
                            }
                            _ => {}
                        },
                    }
                }
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        crossterm::event::MouseEventKind::Down(
                            crossterm::event::MouseButton::Left,
                        ) => {
                            let terminal_size = terminal.size()?;
                            handle_mouse_click(
                                app,
                                mouse_event.column,
                                mouse_event.row,
                                terminal_size.width,
                            );
                        }
                        crossterm::event::MouseEventKind::Moved => {
                            // Track mouse position for hover effects
                            app.mouse_position = Some((mouse_event.column, mouse_event.row));
                            // Update graph cursor if in graph mode
                            if app.state == AppState::Graph {
                                // Reconstruct the same layout used in ui::draw to compute the graph area
                                let size = terminal.size()?;
                                let full = Rect::new(0, 0, size.width, size.height);
                                let v = ratatui::layout::Layout::default()
                                    .direction(ratatui::layout::Direction::Vertical)
                                    .constraints([
                                        ratatui::layout::Constraint::Length(3),
                                        ratatui::layout::Constraint::Min(0),
                                        ratatui::layout::Constraint::Length(3),
                                    ])
                                    .split(full);
                                // In Graph state, ui draws: title, then graph container split into (3, Min, 3)
                                let graph_outer = v[1];
                                let graph_chunks = ratatui::layout::Layout::default()
                                    .direction(ratatui::layout::Direction::Vertical)
                                    .constraints([
                                        ratatui::layout::Constraint::Length(3),
                                        ratatui::layout::Constraint::Min(0),
                                        ratatui::layout::Constraint::Length(3),
                                    ])
                                    .split(graph_outer);
                                let graph_area = graph_chunks[1];
                                app.update_graph_cursor(
                                    mouse_event.column,
                                    mouse_event.row,
                                    graph_area,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
