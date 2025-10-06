use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{App, AppState};

fn create_colored_expression(expression: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let chars = expression.chars();

    for ch in chars {
        let color = match ch {
            '0'..='9' | '.' => Color::White,                        // Numbers
            '+' | '-' | '−' | '*' | '/' | '^' | '%' => Color::Cyan, // Operators
            '(' | ')' => Color::Magenta,                            // Parentheses
            _ => Color::White,                                      // Default
        };

        spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
    }

    spans
}

pub fn draw(f: &mut Frame, app: &App, terminal_size: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status
        ])
        .split(f.area());

    draw_title(f, chunks[0], app);

    match app.state {
        AppState::Graph => draw_graph(f, app, chunks[1], terminal_size),
        _ => {
            if app.show_help {
                draw_help(f, app, chunks[1]);
            } else if app.show_history {
                draw_history(f, app, chunks[1]);
            } else {
                draw_calculator(f, app, chunks[1], terminal_size);
            }
        }
    }

    draw_status(f, app, chunks[2]);
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let mode_str = match app.calculator_module.mode {
        crate::calculator::CalculatorMode::Basic => "Basic",
        crate::calculator::CalculatorMode::Scientific => "Scientific",
    };

    let state_str = match app.state {
        AppState::Normal => {
            if app.second_function_mode {
                "2nd Function Mode"
            } else {
                "Button Navigation"
            }
        }
        AppState::Typing => "Typing Mode",
        AppState::Graph => "Graph Mode",
    };

    let title_text = format!(
        "Calculator | Mode: {} | {} | {}",
        mode_str,
        state_str,
        chrono::Local::now().format("%H:%M:%S")
    );

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_calculator(f: &mut Frame, app: &App, area: Rect, terminal_size: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Display
            Constraint::Min(0),    // Button grid
        ])
        .split(area);

    draw_display(f, app, chunks[0]);
    draw_buttons(f, app, chunks[1], terminal_size);
}

fn draw_display(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Expression
            Constraint::Length(3), // Result
        ])
        .split(area);

    // Expression display with better styling
    let expression = if app.calculator_module.current_expression.is_empty() {
        "0".to_string()
    } else {
        app.calculator_module.current_expression.clone()
    };

    // Create expression spans with right-aligned content
    let mut expression_spans = vec![Span::styled(
        "Expression: ",
        Style::default().fg(Color::Gray),
    )];
    let content_spans = create_colored_expression(&expression);

    // Calculate available width for right-aligned content
    let available_width = chunks[0].width.saturating_sub(14); // 12 for "Expression: " + 2 for borders
    let content_text: String = content_spans
        .iter()
        .map(|span| span.content.clone())
        .collect();

    if content_text.len() <= available_width as usize {
        // Content fits, right-align it with padding
        let padding_needed = available_width.saturating_sub(content_text.len() as u16);
        let padding = " ".repeat(padding_needed as usize);
        expression_spans.push(Span::styled(padding, Style::default()));
        expression_spans.extend(content_spans);
    } else {
        // Content too long, just add it (will overflow gracefully)
        expression_spans.extend(content_spans);
    }

    let expression_para = Paragraph::new(vec![Line::from(expression_spans)]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    f.render_widget(expression_para, chunks[0]);

    // Result display with better styling
    let result_style = if app.calculator_module.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    };

    // Create result spans with right-aligned content
    let mut result_spans = vec![Span::styled("Result: ", Style::default().fg(Color::Gray))];

    // Calculate available width for right-aligned content
    let available_width = chunks[1].width.saturating_sub(10); // 8 for "Result: " + 2 for borders
    let result_text = &app.calculator_module.current_result;

    if result_text.len() <= available_width as usize {
        // Content fits, right-align it with padding
        let padding_needed = available_width.saturating_sub(result_text.len() as u16);
        let padding = " ".repeat(padding_needed as usize);
        result_spans.push(Span::styled(padding, Style::default()));
        result_spans.push(Span::styled(result_text.clone(), result_style));
    } else {
        // Content too long, just add it (will overflow gracefully)
        result_spans.push(Span::styled(result_text.clone(), result_style));
    }

    let result_para = Paragraph::new(vec![Line::from(result_spans)]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(result_para, chunks[1]);
}

fn draw_buttons(f: &mut Frame, app: &App, area: Rect, terminal_size: Rect) {
    let buttons = app.get_calculator_buttons();
    let max_rows = 6; // Maximum visible rows
    let visible_buttons = if buttons.len() > max_rows {
        &buttons[app.scroll_offset..(app.scroll_offset + max_rows).min(buttons.len())]
    } else {
        &buttons
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); visible_buttons.len()])
        .split(area);

    for (row_idx, row) in visible_buttons.iter().enumerate() {
        let row_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25); row.len()])
            .split(chunks[row_idx]);

        for (col_idx, (label, _)) in row.iter().enumerate() {
            let is_selected = if let Some((selected_row, selected_col)) = app.button_position {
                let actual_row = app.scroll_offset + selected_row;
                actual_row == app.scroll_offset + row_idx && selected_col == col_idx
            } else {
                false
            };

            // Check if mouse is hovering over this button
            let is_hovered = if let Some((mouse_x, mouse_y)) = app.mouse_position {
                if let Some((hover_row, hover_col)) =
                    app.mouse_to_button_coords(mouse_x, mouse_y, terminal_size.width)
                {
                    hover_row == row_idx && hover_col == col_idx
                } else {
                    false
                }
            } else {
                false
            };

            // Enhanced button styling with color coding for text and borders only
            let (text_color, border_color, is_bold) = if is_selected || is_hovered {
                // Both selected and hovered use the same yellow highlighting
                (Color::Yellow, Color::Yellow, true)
            } else {
                // Normal colors
                let (color, border) = match *label {
                    "C" | "CE" | "⌫" => (Color::Red, Color::Red), // Clear buttons
                    "=" => (Color::Green, Color::Green),          // Equals
                    "+" | "-" | "−" | "×" | "÷" | "^" | "%" => (Color::Cyan, Color::Cyan), // Operators
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." => {
                        (Color::White, Color::Gray)
                    } // Numbers
                    "(" | ")" => (Color::Magenta, Color::Magenta), // Parentheses
                    "x" | "y" | "z" | "a" | "b" | "c" => (Color::LightGreen, Color::LightGreen), // Variables
                    "π" | "e" => (Color::LightMagenta, Color::LightMagenta), // Constants
                    "2nd" => (Color::LightRed, Color::LightRed),             // 2nd function
                    "Copy" => (Color::Blue, Color::Blue),                    // Copy
                    "Graph" => (Color::Yellow, Color::Yellow),               // Graph
                    // Scientific functions - use distinct colors
                    "sin" | "cos" | "tan" | "√" | "log" | "ln" => {
                        (Color::LightBlue, Color::LightBlue)
                    } // Trig/log functions
                    "exp" | "abs" | "1/x" | "x²" => (Color::Magenta, Color::Magenta), // Advanced functions - same as parentheses
                    _ => (Color::White, Color::Gray),                                 // Fallback
                };
                (color, border, false)
            };

            let text_style = Style::default().fg(text_color).add_modifier(if is_bold {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });

            let button = Paragraph::new(*label)
                .style(text_style)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color)),
                );
            f.render_widget(button, row_chunks[col_idx]);
        }
    }
}

fn draw_history(f: &mut Frame, app: &App, area: Rect) {
    let history_items: Vec<ListItem> = app
        .calculator_module
        .history
        .iter()
        .rev() // Show most recent first
        .enumerate()
        .map(|(idx, entry)| {
            let actual_index = app.calculator_module.history.len() - 1 - idx;
            let is_selected = actual_index == app.history_selected;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();

            let mut history_spans = vec![Span::styled(
                format!("[{}] ", timestamp),
                Style::default().fg(Color::Gray),
            )];
            history_spans.extend(create_colored_expression(&entry.expression));

            ListItem::new(vec![
                Line::from(history_spans),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("= ", Style::default().fg(Color::Gray)),
                    Span::styled(&entry.result, Style::default().fg(Color::Green)),
                ]),
            ])
            .style(style)
        })
        .collect();

    if history_items.is_empty() {
        let empty_widget = Paragraph::new(vec![
            Line::from("No calculations yet"),
            Line::from(""),
            Line::from("Press h to toggle back to calculator"),
            Line::from("Use ↑↓ to navigate history"),
            Line::from("Press r to recall selected entry"),
        ])
        .block(
            Block::default()
                .title("History")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(empty_widget, area);
    } else {
        let history_list = List::new(history_items).block(
            Block::default()
                .title("History (h: back to calc, ↑↓: navigate, r: recall)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(history_list, area);
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let (status_text, status_style) = if let Some(ref error) = app.calculator_module.error_message {
        (
            error.clone(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        (
            app.status_message.clone(),
            Style::default().fg(Color::Yellow),
        )
    };

    let help_text = match app.state {
        AppState::Normal => {
            if app.show_history {
                "h: Back to calculator | ↑↓: Navigate history | r: Recall | q: Quit"
            } else {
                "`: Typing mode | m: Toggle mode | h: History | 2nd: Variables | Ctrl+g: Graph | ←→↑↓: Navigate | Enter/Space/Mouse: Press button | q: Quit"
            }
        }
        AppState::Typing => match app.calculator_module.mode {
            crate::calculator::CalculatorMode::Basic => {
                "Typing Mode: Basic (m: switch to scientific, h: history, Ctrl+g: graph, `: exit, type expressions with variables)"
            }
            crate::calculator::CalculatorMode::Scientific => {
                "Typing Mode: Scientific (m: switch to basic, h: history, Ctrl+g: graph, `: exit, type expressions with variables)"
            }
        },
        AppState::Graph => "Graph Mode: ↑↓←→ pan | +/- zoom | r reset | c toggle coords | Esc exit",
    };

    let status = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(status_text, status_style),
        ]),
        Line::from(vec![
            Span::styled("Help: ", Style::default().fg(Color::Gray)),
            Span::styled(help_text, Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(status, area);
}

fn draw_graph(f: &mut Frame, app: &App, area: Rect, _terminal_size: Rect) {
    // Generate graph points if needed
    if app.graph_module.points.is_empty() {
        // We'll generate points in the main loop, for now just show a placeholder
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Expression display
            Constraint::Min(0),    // Graph area
            Constraint::Length(3), // Controls info
        ])
        .split(area);

    // Draw expression
    let expression_text = format!("f(x) = {}", app.graph_expression);
    let expression_para = Paragraph::new(expression_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
    f.render_widget(expression_para, chunks[0]);

    // Draw graph area
    draw_graph_area(f, app, chunks[1]);

    // Draw controls info
    let controls_text = "Controls: ↑↓←→ pan | +/- zoom | r reset | c toggle coords | Esc exit";
    let controls_para = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    f.render_widget(controls_para, chunks[2]);
}

fn draw_graph_area(f: &mut Frame, app: &App, area: Rect) {
    // Create a simple text-based graph
    let mut graph_lines = Vec::new();

    // Calculate graph dimensions
    let width = area.width as usize;
    let height = area.height as usize;

    // Create a 2D grid to represent the graph
    let mut grid = vec![vec![' '; width]; height];

    // Draw axes
    let x_axis_y = height / 2;
    let y_axis_x = width / 2;

    // Draw x-axis
    for x in 0..width {
        grid[x_axis_y][x] = '─';
    }

    // Draw y-axis
    for row in grid.iter_mut().take(height) {
        row[y_axis_x] = '│';
    }

    // Draw origin
    if x_axis_y < height && y_axis_x < width {
        grid[x_axis_y][y_axis_x] = '┼';
    }

    // Draw graph points
    for point in &app.graph_module.points {
        let x_ratio = (point.x - app.graph_x_min) / (app.graph_x_max - app.graph_x_min);
        let y_ratio = (point.y - app.graph_y_min) / (app.graph_y_max - app.graph_y_min);

        let graph_x = (x_ratio * (width - 1) as f64) as usize;
        let graph_y = ((1.0 - y_ratio) * (height - 1) as f64) as usize;

        if graph_x < width && graph_y < height {
            grid[graph_y][graph_x] = '●';
        }
    }

    // Draw cursor position if enabled
    if app.show_cursor_coords {
        let x_ratio = (app.graph_cursor_x - app.graph_x_min) / (app.graph_x_max - app.graph_x_min);
        let y_ratio = (app.graph_cursor_y - app.graph_y_min) / (app.graph_y_max - app.graph_y_min);

        let cursor_x = (x_ratio * (width - 1) as f64) as usize;
        let cursor_y = ((1.0 - y_ratio) * (height - 1) as f64) as usize;

        if cursor_x < width && cursor_y < height {
            grid[cursor_y][cursor_x] = '×';
        }
    }

    // Convert grid to text lines
    for row in grid {
        let line: String = row.iter().collect();
        graph_lines.push(line);
    }

    // Create the graph widget
    let graph_text = graph_lines.join("\n");
    let graph_para = Paragraph::new(graph_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title("Graph")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(graph_para, area);

    // Draw coordinate info
    if app.show_cursor_coords {
        let coord_text = format!(
            "Cursor: ({:.2}, {:.2}) | Range: x[{:.1}, {:.1}] y[{:.1}, {:.1}]",
            app.graph_cursor_x,
            app.graph_cursor_y,
            app.graph_x_min,
            app.graph_x_max,
            app.graph_y_min,
            app.graph_y_max
        );

        // Draw coordinate info in a small area at the bottom
        let coord_area = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(3),
            area.width,
            3,
        );

        let coord_para = Paragraph::new(coord_text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(coord_para, coord_area);
    }
}

fn draw_help(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = vec![
        "Calculator Help",
        "",
        "Navigation:",
        "  ↑↓←→     Navigate buttons",
        "  Enter    Press selected button",
        "  Space    Press selected button",
        "  Mouse    Click button directly",
        "",
        "Modes:",
        "  `        Toggle typing mode",
        "  m        Switch Basic/Scientific modes",
        "  h        Toggle calculation history",
        "  2nd      Access secondary functions",
        "  ?        Show this help (Esc to close)",
        "",
        "Operations:",
        "  c        Clear current expression",
        "  C        Clear all (expression and history)",
        "  ⌫        Backspace",
        "  r        Recall from history",
        "",
        "Graphing:",
        "  Ctrl+g   Graph current expression (always available)",
        "  Graph    Graph button (2nd function mode only)",
        "  ↑↓←→     Pan graph view",
        "  +/-      Zoom in/out",
        "  r        Reset view to default range",
        "  c        Toggle coordinate display",
        "  Esc      Exit graph mode",
        "",
        "Variables (2nd function mode):",
        "  x, y, z  Primary variables",
        "  a, b, c  Secondary variables",
        "  π        Pi constant (3.14159)",
        "  e        Euler's number (2.71828)",
        "",
        "Scientific Functions (Scientific mode):",
        "  sin, cos, tan  Trigonometric functions",
        "  log, ln        Logarithmic functions",
        "  √, exp         Square root, exponential",
        "  abs, 1/x, x²   Absolute value, reciprocal, square",
        "",
        "Exit:",
        "  q        Quit application",
        "  Esc      Close help or quit",
    ];

    let help_items: Vec<ListItem> = help_text
        .iter()
        .map(|line| {
            let style = if line.is_empty() {
                Style::default()
            } else if line.starts_with("  ") {
                Style::default().fg(Color::White)
            } else if line.ends_with(':') {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            };
            ListItem::new(*line).style(style)
        })
        .collect();

    let help_list = List::new(help_items).block(
        Block::default()
            .title("Help (Press ? or Esc to close)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(help_list, area);
}
