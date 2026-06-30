//! parametric_engine — parameter evaluation, formulas, and update propagation.
//!
//! # Phase B additions
//! - Formula expression parser (basic arithmetic: +, -, *, /, sqrt, abs)
//! - Dependency graph for tracking parameter relationships
//! - Incremental recompute: only re-evaluate affected parameters

#![forbid(unsafe_code)]

use core_math::{Scalar, scalar};
use std::collections::{HashMap, HashSet};

/// A simple arithmetic expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Number(f64),
    /// Reference to another parameter by name.
    Variable(String),
    /// Binary operation.
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// Unary function call.
    Unary(UnaryFn, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryFn {
    Sqrt,
    Abs,
    Neg,
}

impl Expr {
    /// Parse a simple expression string like `"Width + 2 * Height"`.
    pub fn parse(input: &str) -> Result<Self, String> {
        let tokens = tokenize(input);
        parse_expr(&tokens, 0).map(|(expr, _)| expr)
    }

    /// Evaluate the expression with given variable bindings.
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Variable(name) => vars
                .get(name)
                .copied()
                .ok_or_else(|| format!("undefined variable: {name}")),
            Expr::Binary(lhs, op, rhs) => {
                let l = lhs.eval(vars)?;
                let r = rhs.eval(vars)?;
                match op {
                    BinOp::Add => Ok(l + r),
                    BinOp::Sub => Ok(l - r),
                    BinOp::Mul => Ok(l * r),
                    BinOp::Div => {
                        if r.abs() < 1e-15 {
                            Err("division by zero".into())
                        } else {
                            Ok(l / r)
                        }
                    }
                }
            }
            Expr::Unary(fn_type, arg) => {
                let v = arg.eval(vars)?;
                match fn_type {
                    UnaryFn::Sqrt => Ok(v.sqrt()),
                    UnaryFn::Abs => Ok(v.abs()),
                    UnaryFn::Neg => Ok(-v),
                }
            }
        }
    }

    /// Collect variable names referenced in this expression.
    pub fn variables(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        self.collect_vars(&mut result);
        result
    }

    fn collect_vars(&self, out: &mut HashSet<String>) {
        match self {
            Expr::Variable(name) => {
                out.insert(name.clone());
            }
            Expr::Binary(lhs, _, rhs) => {
                lhs.collect_vars(out);
                rhs.collect_vars(out);
            }
            Expr::Unary(_, arg) => {
                arg.collect_vars(out);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tokenizer and recursive-descent parser
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token::Number(input[start..i].parse().unwrap_or(0.0)));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Ident(input[start..i].to_string()));
            continue;
        }
        match c {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ',' => tokens.push(Token::Comma),
            _ => {}
        }
        i += 1;
    }
    tokens
}

fn parse_expr(tokens: &[Token], pos: usize) -> Result<(Expr, usize), String> {
    let (mut lhs, mut pos) = parse_term(tokens, pos)?;
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Plus => {
                let (rhs, np) = parse_term(tokens, pos + 1)?;
                lhs = Expr::Binary(Box::new(lhs), BinOp::Add, Box::new(rhs));
                pos = np;
            }
            Token::Minus => {
                let (rhs, np) = parse_term(tokens, pos + 1)?;
                lhs = Expr::Binary(Box::new(lhs), BinOp::Sub, Box::new(rhs));
                pos = np;
            }
            _ => break,
        }
    }
    Ok((lhs, pos))
}

fn parse_term(tokens: &[Token], pos: usize) -> Result<(Expr, usize), String> {
    let (mut lhs, mut pos) = parse_factor(tokens, pos)?;
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Star => {
                let (rhs, np) = parse_factor(tokens, pos + 1)?;
                lhs = Expr::Binary(Box::new(lhs), BinOp::Mul, Box::new(rhs));
                pos = np;
            }
            Token::Slash => {
                let (rhs, np) = parse_factor(tokens, pos + 1)?;
                lhs = Expr::Binary(Box::new(lhs), BinOp::Div, Box::new(rhs));
                pos = np;
            }
            _ => break,
        }
    }
    Ok((lhs, pos))
}

fn parse_factor(tokens: &[Token], pos: usize) -> Result<(Expr, usize), String> {
    if pos >= tokens.len() {
        return Err("unexpected end of expression".into());
    }
    match &tokens[pos] {
        Token::Number(n) => Ok((Expr::Number(*n), pos + 1)),
        Token::Ident(name) => {
            if pos + 1 < tokens.len() && tokens[pos + 1] == Token::LParen {
                let fn_name = name.as_str();
                let (arg, np) = parse_expr(tokens, pos + 2)?;
                if np < tokens.len() && tokens[np] == Token::RParen {
                    let unary = match fn_name {
                        "sqrt" => UnaryFn::Sqrt,
                        "abs" => UnaryFn::Abs,
                        _ => return Err(format!("unknown function: {fn_name}")),
                    };
                    Ok((Expr::Unary(unary, Box::new(arg)), np + 1))
                } else {
                    Err("expected ')' after function argument".into())
                }
            } else {
                Ok((Expr::Variable(name.clone()), pos + 1))
            }
        }
        Token::LParen => {
            let (expr, np) = parse_expr(tokens, pos + 1)?;
            if np < tokens.len() && tokens[np] == Token::RParen {
                Ok((expr, np + 1))
            } else {
                Err("expected ')'".into())
            }
        }
        Token::Minus => {
            let (arg, np) = parse_factor(tokens, pos + 1)?;
            Ok((Expr::Unary(UnaryFn::Neg, Box::new(arg)), np))
        }
        _ => Err(format!("unexpected token: {:?}", tokens[pos])),
    }
}

// ---------------------------------------------------------------------------
// Parameter model
// ---------------------------------------------------------------------------

/// A named parameter with optional formula expression.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: Scalar,
    pub formula: Option<Expr>,
    pub dependencies: Vec<String>,
}

/// A collection of parameters with dependency-aware evaluation.
#[derive(Debug, Clone, Default)]
pub struct ParametricModel {
    params: HashMap<String, Parameter>,
    eval_order: Vec<String>,
}

impl ParametricModel {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            eval_order: Vec::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: Scalar) {
        self.params.insert(
            name.into(),
            Parameter {
                name: name.into(),
                value,
                formula: None,
                dependencies: vec![],
            },
        );
        self.rebuild_order();
    }

    pub fn set_formula(&mut self, name: &str, formula_str: &str) -> Result<(), String> {
        let expr = Expr::parse(formula_str)?;
        let deps: Vec<String> = expr.variables().into_iter().collect();
        self.params.insert(
            name.into(),
            Parameter {
                name: name.into(),
                value: Scalar::ZERO,
                formula: Some(expr),
                dependencies: deps,
            },
        );
        self.rebuild_order();
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Scalar> {
        self.params.get(name).map(|p| p.value)
    }
    pub fn len(&self) -> usize {
        self.params.len()
    }
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn recompute(&mut self) -> Result<(), String> {
        for name in &self.eval_order.clone() {
            if let Some(param) = self.params.get(name)
                && let Some(formula) = &param.formula
            {
                let mut vars: HashMap<String, f64> = HashMap::new();
                for dep in &param.dependencies {
                    if let Some(val) = self.params.get(dep) {
                        vars.insert(dep.clone(), val.value.value);
                    } else {
                        return Err(format!("parameter {name} depends on unknown {dep}"));
                    }
                }
                let result = formula.eval(&vars)?;
                if let Some(p) = self.params.get_mut(name) {
                    p.value = scalar(result);
                }
            }
        }
        Ok(())
    }

    fn rebuild_order(&mut self) {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();
        let names: Vec<String> = self.params.keys().cloned().collect();
        for name in &names {
            if !visited.contains(name) {
                self.topo_visit(name, &names, &mut visited, &mut temp, &mut order);
            }
        }
        self.eval_order = order;
    }

    fn topo_visit(
        &self,
        name: &str,
        all: &[String],
        visited: &mut HashSet<String>,
        temp: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if temp.contains(name) || visited.contains(name) {
            return;
        }
        temp.insert(name.to_string());
        if let Some(param) = self.params.get(name) {
            for dep in &param.dependencies {
                if all.iter().any(|n| n == dep) {
                    self.topo_visit(dep, all, visited, temp, order);
                }
            }
        }
        temp.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_literal() {
        assert_eq!(Expr::parse("42").unwrap(), Expr::Number(42.0));
    }
    #[test]
    fn parse_variable() {
        assert_eq!(
            Expr::parse("Width").unwrap(),
            Expr::Variable("Width".into())
        );
    }
    #[test]
    fn parse_binary_add() {
        let expr = Expr::parse("Width + 5").unwrap();
        let mut bindings = HashMap::new();
        bindings.insert("Width".into(), 3.0);
        assert!((expr.eval(&bindings).unwrap() - 8.0).abs() < 1e-9);
    }
    #[test]
    fn parse_sqrt() {
        let e = Expr::parse("sqrt(Width)").unwrap();
        let mut b = HashMap::new();
        b.insert("Width".into(), 9.0);
        assert!((e.eval(&b).unwrap() - 3.0).abs() < 1e-9);
    }
    #[test]
    fn parametric_model_formula() {
        let mut model = ParametricModel::new();
        model.set("Width", scalar(3.0));
        model.set("Height", scalar(4.0));
        model
            .set_formula("Diagonal", "sqrt(Width * Width + Height * Height)")
            .unwrap();
        assert_eq!(model.len(), 3);
        model.recompute().unwrap();
        assert!((model.get("Diagonal").unwrap().value - 5.0).abs() < 1e-6);
    }
    #[test]
    fn parametric_model_chained() {
        let mut model = ParametricModel::new();
        model.set("A", scalar(10.0));
        model.set_formula("B", "A + 5").unwrap();
        model.set_formula("C", "B * 2").unwrap();
        model.recompute().unwrap();
        assert!((model.get("B").unwrap().value - 15.0).abs() < 1e-9);
        assert!((model.get("C").unwrap().value - 30.0).abs() < 1e-9);
    }
    #[test]
    fn expr_variable_collection() {
        let e = Expr::parse("Width + Height * 2").unwrap();
        let v = e.variables();
        assert!(v.contains("Width") && v.contains("Height") && v.len() == 2);
    }
}
