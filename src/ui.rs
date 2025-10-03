use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{App, AppState};

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

    if app.show_history {
        draw_history(f, app, chunks[1]);
    } else {
        draw_calculator(f, app, chunks[1], terminal_size);
    }

    draw_status(f, app, chunks[2]);
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let mode_str = match app.calculator_module.mode {
        crate::calculator::CalculatorMode::Basic => "Basic",
        crate::calculator::CalculatorMode::Scientific => "Scientific",
    };

    let state_str = match app.state {
        AppState::Normal => "Button Navigation",
        AppState::Typing => "Typing Mode",
    };

    let title_text = format!(
        "Calculator | Mode: {} | {} | {}",
        mode_str, state_str, 
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

    let expression_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Expression: ", Style::default().fg(Color::Gray)),
            Span::styled(expression, Style::default().fg(Color::White)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
    );
    f.render_widget(expression_para, chunks[0]);

    // Result display with better styling
    let result_style = if app.calculator_module.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };

    let result_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Result: ", Style::default().fg(Color::Gray)),
            Span::styled(app.calculator_module.current_result.clone(), result_style),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
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
                if let Some((hover_row, hover_col)) = app.mouse_to_button_coords(mouse_x, mouse_y, terminal_size.width) {
                    hover_row == row_idx && hover_col == col_idx
                } else {
                    false
                }
            } else {
                false
            };

            // Enhanced button styling with color coding for text and borders only
            let (text_color, border_color) = if is_selected {
                (Color::Yellow, Color::Yellow)
            } else if is_hovered {
                // Hover effect - brighten the colors
                match *label {
                    "C" | "CE" | "⌫" => (Color::Red, Color::Red), // Clear buttons
                    "=" => (Color::Green, Color::Green), // Equals
                    "+" | "-" | "×" | "÷" | "^" | "%" => (Color::Cyan, Color::Cyan), // Operators
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." => (Color::White, Color::White), // Numbers
                    "(" | ")" => (Color::Magenta, Color::Magenta), // Parentheses
                    "Copy" => (Color::Blue, Color::Blue), // Copy
                    _ => (Color::White, Color::White), // Scientific functions
                }
            } else {
                // Normal colors
                match *label {
                    "C" | "CE" | "⌫" => (Color::Red, Color::Red), // Clear buttons
                    "=" => (Color::Green, Color::Green), // Equals
                    "+" | "-" | "×" | "÷" | "^" | "%" => (Color::Cyan, Color::Cyan), // Operators
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." => (Color::White, Color::Gray), // Numbers
                    "(" | ")" => (Color::Magenta, Color::Magenta), // Parentheses
                    "Copy" => (Color::Blue, Color::Blue), // Copy
                    _ => (Color::White, Color::Gray), // Scientific functions
                }
            };

            let text_style = Style::default()
                .fg(text_color)
                .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() });

            let button = Paragraph::new(*label)
                .style(text_style)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
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
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", timestamp),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(&entry.expression, Style::default().fg(Color::White)),
                ]),
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
        let history_list = List::new(history_items)
            .block(
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
        (error.clone(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else {
        (app.status_message.clone(), Style::default().fg(Color::Yellow))
    };

    let help_text = match app.state {
        AppState::Normal => {
            if app.show_history {
                "h: Back to calculator | ↑↓: Navigate history | r: Recall | q: Quit"
            } else {
                "`: Typing mode | m: Toggle mode | h: History | ←→↑↓: Navigate | Enter/Space/Mouse: Press button | q: Quit"
            }
        }
        AppState::Typing => {
            match app.calculator_module.mode {
                crate::calculator::CalculatorMode::Basic => "Typing Mode: Basic (m: switch to scientific, h: history, `: exit, type expressions)",
                crate::calculator::CalculatorMode::Scientific => "Typing Mode: Scientific (m: switch to basic, h: history, `: exit, type expressions)",
            }
        }
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
            .border_style(Style::default().fg(Color::White))
    );

    f.render_widget(status, area);
}