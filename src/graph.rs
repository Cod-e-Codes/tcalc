use anyhow::Result;

#[derive(Debug, Clone)]
pub struct GraphPoint {
    pub x: f64,
    pub y: f64,
}

pub struct GraphModule {
    pub points: Vec<GraphPoint>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Default for GraphModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphModule {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            x_min: -10.0,
            x_max: 10.0,
            y_min: -10.0,
            y_max: 10.0,
        }
    }

    pub fn generate_points(&mut self, expression: &str, width: u16, _height: u16) -> Result<()> {
        self.points.clear();

        let x_range = self.x_max - self.x_min;

        // Generate points for the graph
        for i in 0..width {
            let x = self.x_min + (i as f64 / width as f64) * x_range;

            // Replace 'x' with the current x value in the expression
            let expr_with_x = expression.replace('x', &format!("({})", x));

            match self.evaluate_expression(&expr_with_x) {
                Ok(y) => {
                    // Only add points that are within the y range
                    if y >= self.y_min && y <= self.y_max && y.is_finite() {
                        self.points.push(GraphPoint { x, y });
                    }
                }
                Err(_) => {
                    // Skip invalid points
                    continue;
                }
            }
        }

        Ok(())
    }

    pub fn get_point_at_x(&self, x: f64, expression: &str) -> Option<f64> {
        let expr_with_x = expression.replace('x', &format!("({})", x));
        self.evaluate_expression(&expr_with_x).ok()
    }

    fn evaluate_expression(&self, expr: &str) -> Result<f64> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(0.0);
        }

        let tokens = self.tokenize(expr)?;
        let (result, _) = self.parse_expression(&tokens, 0)?;
        Ok(result)
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = expr.chars().peekable();
        let mut num_buf = String::new();

        while let Some(&ch) = chars.peek() {
            match ch {
                '0'..='9' | '.' => {
                    num_buf.push(ch);
                    chars.next();
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

    fn parse_expression(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
        let (mut left, new_pos) = self.parse_term(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match tokens[pos] {
                Token::Plus => {
                    pos += 1;
                    let (right, next_pos) = self.parse_term(tokens, pos)?;
                    left += right;
                    pos = next_pos;
                }
                Token::Minus => {
                    pos += 1;
                    let (right, next_pos) = self.parse_term(tokens, pos)?;
                    left -= right;
                    pos = next_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    fn parse_term(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
        let (mut left, new_pos) = self.parse_factor(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match tokens[pos] {
                Token::Multiply => {
                    pos += 1;
                    let (right, next_pos) = self.parse_factor(tokens, pos)?;
                    left *= right;
                    pos = next_pos;
                }
                Token::Divide => {
                    pos += 1;
                    let (right, next_pos) = self.parse_factor(tokens, pos)?;
                    if right == 0.0 {
                        return Err(anyhow::anyhow!("Division by zero"));
                    }
                    left /= right;
                    pos = next_pos;
                }
                Token::Modulo => {
                    pos += 1;
                    let (right, next_pos) = self.parse_factor(tokens, pos)?;
                    left %= right;
                    pos = next_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    fn parse_factor(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize)> {
        let (mut base, new_pos) = self.parse_primary(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            if let Token::Power = tokens[pos] {
                pos += 1;
                let (exponent, next_pos) = self.parse_primary(tokens, pos)?;
                base = base.powf(exponent);
                pos = next_pos;
            } else {
                break;
            }
        }

        Ok((base, pos))
    }

    fn parse_primary(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        if pos >= tokens.len() {
            return Err(anyhow::anyhow!("Unexpected end of expression"));
        }

        match &tokens[pos] {
            Token::Number(n) => Ok((*n, pos + 1)),
            Token::Minus => {
                let (value, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((-value, new_pos))
            }
            Token::LParen => {
                let (value, new_pos) = self.parse_expression(tokens, pos + 1)?;
                if new_pos >= tokens.len() || !matches!(tokens[new_pos], Token::RParen) {
                    return Err(anyhow::anyhow!("Missing closing parenthesis"));
                }
                Ok((value, new_pos + 1))
            }
            _ => Err(anyhow::anyhow!("Unexpected token")),
        }
    }
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
}
