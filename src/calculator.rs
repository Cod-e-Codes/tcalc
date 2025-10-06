use anyhow::Result;
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct CalculationEntry {
    pub expression: String,
    pub result: String,
    pub timestamp: DateTime<Local>,
}

pub struct CalculatorModule {
    pub current_expression: String,
    pub current_result: String,
    pub history: Vec<CalculationEntry>,
    pub error_message: Option<String>,
    pub mode: CalculatorMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalculatorMode {
    Basic,
    Scientific,
}

impl Default for CalculatorModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorModule {
    pub fn new() -> Self {
        Self {
            current_expression: String::new(),
            current_result: String::from("0"),
            history: Vec::new(),
            error_message: None,
            mode: CalculatorMode::Basic,
        }
    }

    pub fn append_digit(&mut self, digit: char) {
        self.error_message = None;
        self.current_expression.push(digit);
        self.update_result();
    }

    pub fn append_operator(&mut self, op: &str) {
        self.error_message = None;
        // Allow leading unary minus via buttons
        if self.current_expression.is_empty() {
            if op == "-" {
                self.current_expression.push('-');
            }
            return;
        }
        if !self.current_expression.is_empty() {
            let last_char = self.current_expression.chars().last().unwrap();
            if "+-*/^%".contains(last_char) {
                self.current_expression.pop();
            }
            self.current_expression.push_str(op);
        }
    }

    pub fn append_decimal(&mut self) {
        self.error_message = None;
        let parts: Vec<&str> = self
            .current_expression
            .split(|c: char| "+-*/^%".contains(c))
            .collect();
        if let Some(last_part) = parts.last()
            && !last_part.contains('.')
        {
            if last_part.is_empty() {
                self.current_expression.push_str("0.");
            } else {
                self.current_expression.push('.');
            }
        }
    }

    pub fn backspace(&mut self) {
        self.error_message = None;
        self.current_expression.pop();
        self.update_result();
    }

    pub fn clear(&mut self) {
        self.current_expression.clear();
        self.current_result = String::from("0");
        self.error_message = None;
    }

    pub fn clear_all(&mut self) {
        self.clear();
        self.history.clear();
    }

    pub fn calculate(&mut self) {
        if self.current_expression.is_empty() {
            return;
        }

        match self.evaluate_expression(&self.current_expression) {
            Ok(result) => {
                let result_str = format_result(result);
                self.history.push(CalculationEntry {
                    expression: self.current_expression.clone(),
                    result: result_str.clone(),
                    timestamp: Local::now(),
                });
                self.current_result = result_str.clone();
                self.current_expression = result_str;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
                self.current_result = String::from("Error");
            }
        }
    }

    pub fn update_result(&mut self) {
        if self.current_expression.is_empty() {
            self.current_result = String::from("0");
            return;
        }

        match self.evaluate_expression(&self.current_expression) {
            Ok(result) => {
                self.current_result = format_result(result);
                self.error_message = None;
            }
            Err(_) => {
                self.current_result = self.current_expression.clone();
            }
        }
    }

    pub fn apply_function(&mut self, func: &str) {
        if let Ok(current_val) = self.current_result.parse::<f64>() {
            let result = match func {
                "sin" => current_val.to_radians().sin(),
                "cos" => current_val.to_radians().cos(),
                "tan" => current_val.to_radians().tan(),
                "sqrt" => current_val.sqrt(),
                "log" => current_val.log10(),
                "ln" => current_val.ln(),
                "exp" => current_val.exp(),
                "abs" => current_val.abs(),
                "1/x" => 1.0 / current_val,
                "x^2" => current_val.powi(2),
                _ => return,
            };

            let result_str = format_result(result);
            self.history.push(CalculationEntry {
                expression: format!("{}({})", func, current_val),
                result: result_str.clone(),
                timestamp: Local::now(),
            });
            self.current_expression = result_str.clone();
            self.current_result = result_str;
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            CalculatorMode::Basic => CalculatorMode::Scientific,
            CalculatorMode::Scientific => CalculatorMode::Basic,
        };
    }

    pub fn recall_from_history(&mut self, index: usize) {
        if index < self.history.len() {
            // Recall the original expression, then update the live result
            self.current_expression = self.history[index].expression.clone();
            match self.evaluate_expression(&self.current_expression) {
                Ok(result) => {
                    self.current_result = format_result(result);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Error: {}", e));
                    self.current_result = String::from("Error");
                }
            }
        }
    }

    pub fn copy_result_to_clipboard(&self) -> Result<String> {
        Ok(self.current_result.clone())
    }

    fn evaluate_expression(&self, expr: &str) -> Result<f64> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(0.0);
        }

        let tokens = tokenize(expr)?;
        let (result, _) = parse_expression(&tokens, 0)?;
        Ok(result)
    }
}

fn format_result(value: f64) -> String {
    if value.is_infinite() {
        return "Infinity".to_string();
    }
    if value.is_nan() {
        return "NaN".to_string();
    }

    let s = format!("{:.10}", value);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn tokenize(expr: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    let mut num_buf = String::new();
    let mut ident_buf = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | '.' => {
                num_buf.push(ch);
                chars.next();
            }
            'a'..='z' | 'A'..='Z' | 'π' => {
                // flush number buffer
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse()?));
                    num_buf.clear();
                }
                ident_buf.push(ch);
                chars.next();
                // collect full identifier
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        ident_buf.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let ident = ident_buf.to_lowercase();
                ident_buf.clear();
                match ident.as_str() {
                    // constants
                    "pi" | "π" => tokens.push(Token::Number(std::f64::consts::PI)),
                    "e" => tokens.push(Token::Number(std::f64::consts::E)),
                    // recognized function names become identifiers; parsing will handle call
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            '+' | '-' | '*' | '/' | '^' | '%' | '(' | ')' => {
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse()?));
                    num_buf.clear();
                }
                tokens.push(match ch {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '^' => Token::Power,
                    '%' => Token::Modulo,
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    _ => unreachable!(),
                });
                chars.next();
            }
            ' ' => {
                chars.next();
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid character: {}", ch));
            }
        }
    }

    if !num_buf.is_empty() {
        tokens.push(Token::Number(num_buf.parse()?));
    }

    // Add implicit multiplication tokens
    let mut result = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        result.push(token.clone());

        // Check if we need to add implicit multiplication
        if i < tokens.len() - 1 {
            match (token, &tokens[i + 1]) {
                // Number followed by opening parenthesis: 3( -> 3*(
                (Token::Number(_), Token::LParen) => {
                    result.push(Token::Multiply);
                }
                // Closing parenthesis followed by number: )3 -> )*3
                (Token::RParen, Token::Number(_)) => {
                    result.push(Token::Multiply);
                }
                // Closing parenthesis followed by opening parenthesis: )( -> )*(
                (Token::RParen, Token::LParen) => {
                    result.push(Token::Multiply);
                }
                _ => {}
            }
        }
    }

    Ok(result)
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Modulo,
    LParen,
    RParen,
    Ident(String),
}

fn parse_expression(tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
    let (mut left, new_pos) = parse_term(tokens, pos)?;
    pos = new_pos;

    while pos < tokens.len() {
        match tokens[pos] {
            Token::Plus => {
                pos += 1;
                let (right, next_pos) = parse_term(tokens, pos)?;
                left += right;
                pos = next_pos;
            }
            Token::Minus => {
                pos += 1;
                let (right, next_pos) = parse_term(tokens, pos)?;
                left -= right;
                pos = next_pos;
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

fn parse_term(tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
    let (mut left, new_pos) = parse_factor(tokens, pos)?;
    pos = new_pos;

    while pos < tokens.len() {
        match tokens[pos] {
            Token::Multiply => {
                pos += 1;
                let (right, next_pos) = parse_factor(tokens, pos)?;
                left *= right;
                pos = next_pos;
            }
            Token::Divide => {
                pos += 1;
                let (right, next_pos) = parse_factor(tokens, pos)?;
                if right == 0.0 {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
                left /= right;
                pos = next_pos;
            }
            Token::Modulo => {
                pos += 1;
                let (right, next_pos) = parse_factor(tokens, pos)?;
                left %= right;
                pos = next_pos;
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

fn parse_factor(tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
    let (mut base, new_pos) = parse_primary(tokens, pos)?;
    pos = new_pos;

    while pos < tokens.len() {
        if let Token::Power = tokens[pos] {
            pos += 1;
            let (exponent, next_pos) = parse_primary(tokens, pos)?;
            base = base.powf(exponent);
            pos = next_pos;
        } else {
            break;
        }
    }

    Ok((base, pos))
}

fn parse_primary(tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
    if pos >= tokens.len() {
        return Err(anyhow::anyhow!("Unexpected end of expression"));
    }

    match &tokens[pos] {
        Token::Number(n) => Ok((*n, pos + 1)),
        Token::Minus => {
            let (value, new_pos) = parse_primary(tokens, pos + 1)?;
            Ok((-value, new_pos))
        }
        Token::LParen => {
            let (value, new_pos) = parse_expression(tokens, pos + 1)?;
            if new_pos >= tokens.len() || !matches!(tokens[new_pos], Token::RParen) {
                return Err(anyhow::anyhow!("Missing closing parenthesis"));
            }
            Ok((value, new_pos + 1))
        }
        Token::Ident(name) => {
            // function call: ident '(' expr ')'
            if pos + 1 < tokens.len() && matches!(tokens[pos + 1], Token::LParen) {
                let (arg, np) = parse_expression(tokens, pos + 2)?; // skip ident + '('
                if np >= tokens.len() || !matches!(tokens[np], Token::RParen) {
                    return Err(anyhow::anyhow!("Missing closing parenthesis"));
                }
                let val = match name.as_str() {
                    "sin" => arg.sin(),
                    "cos" => arg.cos(),
                    "tan" => arg.tan(),
                    "sqrt" => arg.sqrt(),
                    "log" => arg.log10(),
                    "ln" => arg.ln(),
                    "exp" => arg.exp(),
                    "abs" => arg.abs(),
                    _ => return Err(anyhow::anyhow!("Unknown function: {}", name)),
                };
                Ok((val, np + 1))
            } else {
                Err(anyhow::anyhow!("Unexpected identifier: {}", name))
            }
        }
        _ => Err(anyhow::anyhow!("Unexpected token")),
    }
}
