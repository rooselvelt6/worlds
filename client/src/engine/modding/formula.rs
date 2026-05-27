use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Func { name: String, args: Vec<Expr> },
}

fn parse_atom(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".into());
    }
    let tok = &tokens[*pos];
    *pos += 1;
    if let Ok(n) = tok.parse::<f64>() {
        return Ok(Expr::Num(n));
    }
    if tok == "(" {
        let e = parse_expr(tokens, pos)?;
        if *pos >= tokens.len() || tokens[*pos] != ")" {
            return Err("Expected ')'".into());
        }
        *pos += 1;
        return Ok(e);
    }
    if tok == "-" {
        let e = parse_atom(tokens, pos)?;
        return Ok(Expr::Neg(Box::new(e)));
    }
    if tok == "+" {
        return parse_atom(tokens, pos);
    }
    // Function call or variable
    if *pos < tokens.len() && tokens[*pos] == "(" {
        let name = tok.clone();
        *pos += 1;
        let mut args = Vec::new();
        if *pos < tokens.len() && tokens[*pos] != ")" {
            args.push(parse_expr(tokens, pos)?);
            while *pos < tokens.len() && tokens[*pos] == "," {
                *pos += 1;
                args.push(parse_expr(tokens, pos)?);
            }
        }
        if *pos >= tokens.len() || tokens[*pos] != ")" {
            return Err("Expected ')' after function arguments".into());
        }
        *pos += 1;
        return Ok(Expr::Func { name, args });
    }
    // Variable
    Ok(Expr::Var(tok.clone()))
}

fn parse_pow(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_atom(tokens, pos)?;
    while *pos < tokens.len() && tokens[*pos] == "^" {
        *pos += 1;
        let right = parse_atom(tokens, pos)?;
        left = Expr::Pow(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_unary(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end".into());
    }
    if tokens[*pos] == "-" {
        *pos += 1;
        let e = parse_unary(tokens, pos)?;
        return Ok(Expr::Neg(Box::new(e)));
    }
    if tokens[*pos] == "+" {
        *pos += 1;
        return parse_unary(tokens, pos);
    }
    parse_pow(tokens, pos)
}

fn parse_mul(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_unary(tokens, pos)?;
    while *pos < tokens.len() {
        let op = &tokens[*pos];
        if op == "*" || op == "/" {
            *pos += 1;
            let right = parse_unary(tokens, pos)?;
            if op == "*" {
                left = Expr::Mul(Box::new(left), Box::new(right));
            } else {
                left = Expr::Div(Box::new(left), Box::new(right));
            }
        } else {
            break;
        }
    }
    Ok(left)
}

fn parse_expr(tokens: &[String], pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_mul(tokens, pos)?;
    while *pos < tokens.len() {
        let op = &tokens[*pos];
        if op == "+" || op == "-" {
            *pos += 1;
            let right = parse_mul(tokens, pos)?;
            if op == "+" {
                left = Expr::Add(Box::new(left), Box::new(right));
            } else {
                left = Expr::Sub(Box::new(left), Box::new(right));
            }
        } else {
            break;
        }
    }
    Ok(left)
}

pub fn parse(expr: &str) -> Result<Expr, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            current.clear();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                current.push(chars[i]);
                i += 1;
            }
            tokens.push(current.clone());
        } else if c.is_ascii_alphabetic() || c == '_' {
            current.clear();
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                current.push(chars[i]);
                i += 1;
            }
            tokens.push(current.clone());
        } else if "+-*/^(),".contains(c) {
            tokens.push(c.to_string());
            i += 1;
        } else if c == '(' || c == ')' {
            tokens.push(c.to_string());
            i += 1;
        } else {
            return Err(format!("Unexpected character: '{}'", c));
        }
    }
    let mut pos = 0;
    let ast = parse_expr(&tokens, &mut pos)?;
    if pos < tokens.len() {
        return Err(format!("Unexpected token '{}' after expression", tokens[pos]));
    }
    Ok(ast)
}

pub fn eval(expr: &Expr, vars: &HashMap<String, f64>) -> Result<f64, String> {
    match expr {
        Expr::Num(n) => Ok(*n),
        Expr::Var(name) => {
            let lower = name.to_lowercase();
            match lower.as_str() {
                "pi" => Ok(std::f64::consts::PI),
                "e" => Ok(std::f64::consts::E),
                _ => vars.get(&lower).copied().ok_or_else(|| format!("Unknown variable: '{}'", name)),
            }
        }
        Expr::Neg(e) => Ok(-eval(e, vars)?),
        Expr::Add(a, b) => Ok(eval(a, vars)? + eval(b, vars)?),
        Expr::Sub(a, b) => Ok(eval(a, vars)? - eval(b, vars)?),
        Expr::Mul(a, b) => Ok(eval(a, vars)? * eval(b, vars)?),
        Expr::Div(a, b) => {
            let denom = eval(b, vars)?;
            if denom == 0.0 { return Err("Division by zero".into()); }
            Ok(eval(a, vars)? / denom)
        }
        Expr::Pow(a, b) => Ok(eval(a, vars)?.powf(eval(b, vars)?)),
        Expr::Func { name, args } => {
            let lower = name.to_lowercase();
            match lower.as_str() {
                "sin" => {
                    if args.len() != 1 { return Err("sin requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.sin())
                }
                "cos" => {
                    if args.len() != 1 { return Err("cos requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.cos())
                }
                "abs" => {
                    if args.len() != 1 { return Err("abs requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.abs())
                }
                "sqrt" => {
                    if args.len() != 1 { return Err("sqrt requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.sqrt())
                }
                "floor" => {
                    if args.len() != 1 { return Err("floor requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.floor())
                }
                "ceil" => {
                    if args.len() != 1 { return Err("ceil requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.ceil())
                }
                "round" => {
                    if args.len() != 1 { return Err("round requires 1 argument".into()); }
                    Ok(eval(&args[0], vars)?.round())
                }
                "min" => {
                    if args.len() != 2 { return Err("min requires 2 arguments".into()); }
                    Ok(eval(&args[0], vars)?.min(eval(&args[1], vars)?))
                }
                "max" => {
                    if args.len() != 2 { return Err("max requires 2 arguments".into()); }
                    Ok(eval(&args[0], vars)?.max(eval(&args[1], vars)?))
                }
                "clamp" => {
                    if args.len() != 3 { return Err("clamp requires 3 arguments".into()); }
                    let val = eval(&args[0], vars)?;
                    let lo = eval(&args[1], vars)?;
                    let hi = eval(&args[2], vars)?;
                    Ok(val.clamp(lo, hi))
                }
                "lerp" => {
                    if args.len() != 3 { return Err("lerp requires 3 arguments".into()); }
                    let a = eval(&args[0], vars)?;
                    let b = eval(&args[1], vars)?;
                    let t = eval(&args[2], vars)?;
                    Ok(a + (b - a) * t)
                }
                _ => Err(format!("Unknown function: '{}'", name)),
            }
        }
    }
}

pub fn eval_str(expr_str: &str, vars: &HashMap<String, f64>) -> Result<f64, String> {
    let ast = parse(expr_str)?;
    eval(&ast, vars)
}
