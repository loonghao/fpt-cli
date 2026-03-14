use fpt_core::{AppError, Result};
use serde_json::{json, Number, Value};

#[derive(Debug, Clone, PartialEq)]
enum LogicalOp {
    And,
    Or,
}

impl LogicalOp {
    fn as_str(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Comparison {
        field: String,
        operator: String,
        value: Value,
    },
    Logical {
        op: LogicalOp,
        conditions: Vec<Expr>,
    },
}

pub fn parse_filter_dsl(input: &str) -> Result<Value> {
    let source = input.trim();
    if source.is_empty() {
        return Err(AppError::invalid_input("`filter_dsl` 不能为空"));
    }

    let mut parser = Parser::new(source);
    let expr = parser.parse_expression()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
        return parser.error("存在无法解析的多余内容");
    }

    Ok(wrap_root(expr))
}

fn wrap_root(expr: Expr) -> Value {
    match expr {
        Expr::Logical { .. } => expr_to_value(expr),
        value => json!({
            "logical_operator": "and",
            "conditions": [expr_to_value(value)],
        }),
    }
}

fn expr_to_value(expr: Expr) -> Value {
    match expr {
        Expr::Comparison {
            field,
            operator,
            value,
        } => json!([field, operator, value]),
        Expr::Logical { op, conditions } => json!({
            "logical_operator": op.as_str(),
            "conditions": conditions.into_iter().map(expr_to_value).collect::<Vec<_>>(),
        }),
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    _input: &'a str,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _input: input,
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut terms = vec![self.parse_and()?];

        loop {
            let checkpoint = self.pos;
            if self.consume_keyword("or") {
                terms.push(self.parse_and()?);
            } else {
                self.pos = checkpoint;
                break;
            }
        }

        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            Ok(Expr::Logical {
                op: LogicalOp::Or,
                conditions: terms,
            })
        }
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut terms = vec![self.parse_primary()?];

        loop {
            let checkpoint = self.pos;
            if self.consume_keyword("and") {
                terms.push(self.parse_primary()?);
            } else {
                self.pos = checkpoint;
                break;
            }
        }

        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            Ok(Expr::Logical {
                op: LogicalOp::And,
                conditions: terms,
            })
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_whitespace();
        if self.consume_char('(') {
            let expr = self.parse_expression()?;
            self.skip_whitespace();
            if !self.consume_char(')') {
                return self.error("缺少右括号 `)`");
            }
            return Ok(expr);
        }

        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let field = self.parse_field_path()?;
        let operator = self.parse_operator()?;
        let value = self.parse_value()?;

        Ok(Expr::Comparison {
            field,
            operator,
            value,
        })
    }

    fn parse_field_path(&mut self) -> Result<String> {
        let mut path = String::new();
        path.push_str(&self.parse_identifier("字段名")?);

        loop {
            self.skip_whitespace();
            if !self.consume_char('.') {
                break;
            }
            path.push('.');
            path.push_str(&self.parse_identifier("字段名")?);
        }

        Ok(path)
    }

    fn parse_operator(&mut self) -> Result<String> {
        self.skip_whitespace();

        if self.consume_literal("==") {
            return Ok("is".to_string());
        }
        if self.consume_literal("!=") {
            return Ok("is_not".to_string());
        }
        if self.consume_literal(">=") {
            return Ok("greater_than_or_equal".to_string());
        }
        if self.consume_literal("<=") {
            return Ok("less_than_or_equal".to_string());
        }
        if self.consume_literal(">") {
            return Ok("greater_than".to_string());
        }
        if self.consume_literal("<") {
            return Ok("less_than".to_string());
        }
        if self.consume_literal("~") {
            return Ok("contains".to_string());
        }

        let keyword = self.parse_identifier("操作符")?.to_ascii_lowercase();
        if keyword == "not" {
            let checkpoint = self.pos;
            if self.consume_keyword("in") {
                return Ok("not_in".to_string());
            }
            self.pos = checkpoint;
        }

        Ok(keyword)
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace();
        let Some(ch) = self.peek_char() else {
            return self.error("缺少操作数");
        };

        match ch {
            '"' | '\'' => self.parse_string().map(Value::String),
            '[' => self.parse_array(),
            '-' | '0'..='9' => self.parse_number(),
            _ => {
                let ident = self.parse_identifier("值")?.to_ascii_lowercase();
                match ident.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    "null" => Ok(Value::Null),
                    _ => Err(AppError::invalid_input(format!(
                        "filter_dsl 语法错误: 不支持的值 `{ident}`，请使用字符串/数字/布尔/null（位置 {}）",
                        self.pos
                    ))),
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.expect_char('[', "数组起始 `[`")?;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.consume_char(']') {
            return Ok(Value::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();

            if self.consume_char(']') {
                break;
            }
            self.expect_char(',', "数组分隔符 `,`")?;
        }

        Ok(Value::Array(values))
    }

    fn parse_number(&mut self) -> Result<Value> {
        self.skip_whitespace();
        let mut literal = String::new();

        if self.consume_char('-') {
            literal.push('-');
        }

        let mut has_integer = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                has_integer = true;
                literal.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        if !has_integer {
            return self.error("数字格式不正确");
        }

        if self.consume_char('.') {
            literal.push('.');
            let mut has_fraction = false;
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    has_fraction = true;
                    literal.push(ch);
                    self.pos += 1;
                } else {
                    break;
                }
            }

            if !has_fraction {
                return self.error("浮点数格式不正确");
            }

            let value = literal
                .parse::<f64>()
                .map_err(|error| AppError::invalid_input(format!("数字解析失败: {error}")))?;
            let number = Number::from_f64(value)
                .ok_or_else(|| AppError::invalid_input("不支持 NaN 或 Infinity"))?;
            return Ok(Value::Number(number));
        }

        let value = literal
            .parse::<i64>()
            .map_err(|error| AppError::invalid_input(format!("数字解析失败: {error}")))?;
        Ok(Value::Number(Number::from(value)))
    }

    fn parse_string(&mut self) -> Result<String> {
        self.skip_whitespace();
        let quote = self
            .peek_char()
            .ok_or_else(|| AppError::invalid_input("filter_dsl 语法错误: 缺少字符串开头引号"))?;
        if !matches!(quote, '"' | '\'') {
            return self.error("字符串必须以引号包裹");
        }
        self.pos += 1;

        let mut output = String::new();
        while let Some(ch) = self.peek_char() {
            self.pos += 1;
            if ch == quote {
                return Ok(output);
            }

            if ch == '\\' {
                let escaped = self
                    .peek_char()
                    .ok_or_else(|| AppError::invalid_input("filter_dsl 语法错误: 非法转义结尾"))?;
                self.pos += 1;
                let normalized = match escaped {
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => {
                        return Err(AppError::invalid_input(format!(
                            "filter_dsl 语法错误: 不支持的转义字符 `\\{other}`（位置 {}）",
                            self.pos
                        )))
                    }
                };
                output.push(normalized);
                continue;
            }

            output.push(ch);
        }

        self.error("字符串缺少结束引号")
    }

    fn parse_identifier(&mut self, label: &str) -> Result<String> {
        self.skip_whitespace();

        let Some(first) = self.peek_char() else {
            return self.error(format!("缺少{label}"));
        };

        if !is_identifier_start(first) {
            return self.error(format!("{label}必须以字母或下划线开头"));
        }

        let mut output = String::new();
        output.push(first);
        self.pos += 1;

        while let Some(ch) = self.peek_char() {
            if is_identifier_continue(ch) {
                output.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(output)
    }

    fn expect_char(&mut self, expected: char, message: &str) -> Result<()> {
        self.skip_whitespace();
        if self.consume_char(expected) {
            Ok(())
        } else {
            self.error(format!("缺少{message}"))
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let checkpoint = self.pos;

        for expected in keyword.chars() {
            match self.peek_char() {
                Some(ch) if ch.eq_ignore_ascii_case(&expected) => self.pos += 1,
                _ => {
                    self.pos = checkpoint;
                    return false;
                }
            }
        }

        if self.peek_char().is_some_and(is_identifier_continue) {
            self.pos = checkpoint;
            return false;
        }

        true
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        self.skip_whitespace();
        let checkpoint = self.pos;
        for expected in literal.chars() {
            match self.peek_char() {
                Some(ch) if ch == expected => self.pos += 1,
                _ => {
                    self.pos = checkpoint;
                    return false;
                }
            }
        }
        true
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(|ch| ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(AppError::invalid_input(format!(
            "filter_dsl 语法错误: {}（位置 {}）",
            message.into(),
            self.pos
        )))
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
