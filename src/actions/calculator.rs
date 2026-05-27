//! Inline calculator - math, unit conversion, hex/binary.

/// Try to evaluate the input as a math expression or conversion.
/// Returns None if input doesn't look like math.
pub fn evaluate(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() { return None; }

    // Hex conversion: "0xff" or "0xFF" or "255 in hex"
    if let Some(result) = try_hex_conversion(input) {
        return Some(result);
    }

    // Unit conversion: "100f to c", "32gb in mb"
    if let Some(result) = try_unit_conversion(input) {
        return Some(result);
    }

    // Math expression
    try_math(input)
}

fn try_math(input: &str) -> Option<String> {
    // Only evaluate if it contains math-like characters
    if !input.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')')) {
        // Could be a plain number
        if input.parse::<f64>().is_ok() {
            return None; // Just a number, not interesting
        }
        return None;
    }

    // Simple expression evaluator
    let result = eval_expr(input)?;

    // Format nicely
    if result == result.floor() && result.abs() < 1e15 {
        Some(format!("= {}", result as i64))
    } else {
        Some(format!("= {:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string())
    }
}

/// Simple recursive descent expression parser.
/// Supports: + - * / % ^ ( ) and decimal numbers.
fn eval_expr(input: &str) -> Option<f64> {
    let tokens: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut pos = 0;
    let result = parse_add(&tokens, &mut pos)?;
    if pos == tokens.len() { Some(result) } else { None }
}

fn parse_add(tokens: &[char], pos: &mut usize) -> Option<f64> {
    let mut left = parse_mul(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            '+' => { *pos += 1; left += parse_mul(tokens, pos)?; }
            '-' => { *pos += 1; left -= parse_mul(tokens, pos)?; }
            _ => break,
        }
    }
    Some(left)
}

fn parse_mul(tokens: &[char], pos: &mut usize) -> Option<f64> {
    let mut left = parse_pow(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            '*' => { *pos += 1; left *= parse_pow(tokens, pos)?; }
            '/' => { *pos += 1; let r = parse_pow(tokens, pos)?; if r == 0.0 { return None; } left /= r; }
            '%' => { *pos += 1; let r = parse_pow(tokens, pos)?; if r == 0.0 { return None; } left %= r; }
            _ => break,
        }
    }
    Some(left)
}

fn parse_pow(tokens: &[char], pos: &mut usize) -> Option<f64> {
    let base = parse_unary(tokens, pos)?;
    if *pos < tokens.len() && tokens[*pos] == '^' {
        *pos += 1;
        let exp = parse_pow(tokens, pos)?;
        Some(base.powf(exp))
    } else {
        Some(base)
    }
}

fn parse_unary(tokens: &[char], pos: &mut usize) -> Option<f64> {
    if *pos < tokens.len() && tokens[*pos] == '-' {
        *pos += 1;
        Some(-parse_atom(tokens, pos)?)
    } else {
        parse_atom(tokens, pos)
    }
}

fn parse_atom(tokens: &[char], pos: &mut usize) -> Option<f64> {
    if *pos >= tokens.len() { return None; }

    if tokens[*pos] == '(' {
        *pos += 1;
        let val = parse_add(tokens, pos)?;
        if *pos < tokens.len() && tokens[*pos] == ')' {
            *pos += 1;
        }
        return Some(val);
    }

    // Parse number
    let start = *pos;
    while *pos < tokens.len() && (tokens[*pos].is_ascii_digit() || tokens[*pos] == '.') {
        *pos += 1;
    }
    if *pos == start { return None; }

    let num_str: String = tokens[start..*pos].iter().collect();
    num_str.parse().ok()
}

fn try_hex_conversion(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // "0xff" -> decimal
    if let Some(hex) = lower.strip_prefix("0x") {
        let val = u64::from_str_radix(hex, 16).ok()?;
        return Some(format!("= {} (decimal)", val));
    }

    // "255 in hex" or "255 to hex"
    if lower.ends_with(" in hex") || lower.ends_with(" to hex") {
        let num_str = lower.split_whitespace().next()?;
        let val: u64 = num_str.parse().ok()?;
        return Some(format!("= 0x{:X}", val));
    }

    // "255 in binary" or "255 to binary" or "255 in bin"
    if lower.ends_with(" in binary") || lower.ends_with(" to binary") || lower.ends_with(" in bin") {
        let num_str = lower.split_whitespace().next()?;
        let val: u64 = num_str.parse().ok()?;
        return Some(format!("= {:b}", val));
    }

    // "0b1111" -> decimal
    if let Some(bin) = lower.strip_prefix("0b") {
        let val = u64::from_str_radix(bin, 2).ok()?;
        return Some(format!("= {} (decimal)", val));
    }

    None
}

fn try_unit_conversion(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let parts: Vec<&str> = lower.split(' ').collect();

    // Pattern: "VALUE UNIT to/in UNIT"
    if parts.len() < 4 { return None; }
    let val: f64 = parts[0].parse().ok()?;
    let from = parts[1];
    // parts[2] should be "to" or "in"
    if parts[2] != "to" && parts[2] != "in" { return None; }
    let to = parts[3];

    // Temperature
    match (from, to) {
        ("f" | "fahrenheit", "c" | "celsius") => return Some(format!("= {:.1}C", (val - 32.0) * 5.0 / 9.0)),
        ("c" | "celsius", "f" | "fahrenheit") => return Some(format!("= {:.1}F", val * 9.0 / 5.0 + 32.0)),
        ("k" | "kelvin", "c" | "celsius") => return Some(format!("= {:.1}C", val - 273.15)),
        ("c" | "celsius", "k" | "kelvin") => return Some(format!("= {:.1}K", val + 273.15)),
        _ => {}
    }

    // Data sizes
    let bytes = match from {
        "b" | "bytes" => Some(val),
        "kb" => Some(val * 1024.0),
        "mb" => Some(val * 1024.0 * 1024.0),
        "gb" => Some(val * 1024.0 * 1024.0 * 1024.0),
        "tb" => Some(val * 1024.0 * 1024.0 * 1024.0 * 1024.0),
        _ => None,
    };

    if let Some(b) = bytes {
        let result = match to {
            "b" | "bytes" => Some(format!("= {:.0} B", b)),
            "kb" => Some(format!("= {:.2} KB", b / 1024.0)),
            "mb" => Some(format!("= {:.2} MB", b / (1024.0 * 1024.0))),
            "gb" => Some(format!("= {:.2} GB", b / (1024.0 * 1024.0 * 1024.0))),
            "tb" => Some(format!("= {:.4} TB", b / (1024.0 * 1024.0 * 1024.0 * 1024.0))),
            _ => None,
        };
        if result.is_some() { return result; }
    }

    // Distance
    let meters = match from {
        "km" | "kilometers" => Some(val * 1000.0),
        "m" | "meters" => Some(val),
        "cm" | "centimeters" => Some(val / 100.0),
        "mi" | "miles" => Some(val * 1609.34),
        "ft" | "feet" => Some(val * 0.3048),
        "in" | "inches" if to != "hex" && to != "binary" && to != "bin" => Some(val * 0.0254),
        _ => None,
    };

    if let Some(m) = meters {
        let result = match to {
            "km" | "kilometers" => Some(format!("= {:.4} km", m / 1000.0)),
            "m" | "meters" => Some(format!("= {:.2} m", m)),
            "cm" | "centimeters" => Some(format!("= {:.1} cm", m * 100.0)),
            "mi" | "miles" => Some(format!("= {:.4} mi", m / 1609.34)),
            "ft" | "feet" => Some(format!("= {:.2} ft", m / 0.3048)),
            "inches" => Some(format!("= {:.2} in", m / 0.0254)),
            _ => None,
        };
        if result.is_some() { return result; }
    }

    // Weight
    let grams = match from {
        "kg" | "kilograms" => Some(val * 1000.0),
        "g" | "grams" => Some(val),
        "lb" | "lbs" | "pounds" => Some(val * 453.592),
        "oz" | "ounces" => Some(val * 28.3495),
        _ => None,
    };

    if let Some(g) = grams {
        let result = match to {
            "kg" | "kilograms" => Some(format!("= {:.4} kg", g / 1000.0)),
            "g" | "grams" => Some(format!("= {:.2} g", g)),
            "lb" | "lbs" | "pounds" => Some(format!("= {:.4} lbs", g / 453.592)),
            "oz" | "ounces" => Some(format!("= {:.2} oz", g / 28.3495)),
            _ => None,
        };
        if result.is_some() { return result; }
    }

    None
}
