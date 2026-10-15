//! Inline calculator: arithmetic, hex/binary, and unit conversion.

/// Evaluates `input` if it looks like math or a conversion. The result is the
/// bare value, ready to copy (`"1024"`, `"0xFF"`, `"100 °C"`).
pub fn evaluate(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    convert_base(input)
        .or_else(|| convert_unit(input))
        .or_else(|| arithmetic(input))
}

fn arithmetic(input: &str) -> Option<String> {
    // A bare number is a search, not a calculation.
    if !input.contains(['+', '-', '*', '/', '%', '^', '(', ')']) {
        return None;
    }
    let value = eval(input)?;
    value.is_finite().then(|| format_number(value))
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        let fixed = format!("{value:.6}");
        fixed
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// Recursive descent over `+ - * / % ^ ( )` and decimal numbers.
fn eval(input: &str) -> Option<f64> {
    let tokens: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut pos = 0;
    let value = sum(&tokens, &mut pos)?;
    (pos == tokens.len()).then_some(value)
}

fn sum(t: &[char], pos: &mut usize) -> Option<f64> {
    let mut left = product(t, pos)?;
    while let Some(&op) = t.get(*pos) {
        match op {
            '+' => {
                *pos += 1;
                left += product(t, pos)?;
            }
            '-' => {
                *pos += 1;
                left -= product(t, pos)?;
            }
            _ => break,
        }
    }
    Some(left)
}

fn product(t: &[char], pos: &mut usize) -> Option<f64> {
    let mut left = power(t, pos)?;
    while let Some(&op) = t.get(*pos) {
        if !matches!(op, '*' | '/' | '%') {
            break;
        }
        *pos += 1;
        let right = power(t, pos)?;
        left = match op {
            '*' => left * right,
            _ if right == 0.0 => return None,
            '/' => left / right,
            _ => left % right,
        };
    }
    Some(left)
}

fn power(t: &[char], pos: &mut usize) -> Option<f64> {
    let base = unary(t, pos)?;
    if t.get(*pos) == Some(&'^') {
        *pos += 1;
        return Some(base.powf(power(t, pos)?));
    }
    Some(base)
}

fn unary(t: &[char], pos: &mut usize) -> Option<f64> {
    if t.get(*pos) == Some(&'-') {
        *pos += 1;
        return Some(-unary(t, pos)?);
    }
    atom(t, pos)
}

fn atom(t: &[char], pos: &mut usize) -> Option<f64> {
    if t.get(*pos) == Some(&'(') {
        *pos += 1;
        let value = sum(t, pos)?;
        if t.get(*pos) == Some(&')') {
            *pos += 1;
        }
        return Some(value);
    }
    let start = *pos;
    while t.get(*pos).is_some_and(|c| c.is_ascii_digit() || *c == '.') {
        *pos += 1;
    }
    t[start..*pos].iter().collect::<String>().parse().ok()
}

/// `0xff`, `0b1010`, `255 in hex`, `10 to binary`.
fn convert_base(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    if let Some(hex) = lower.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16).ok().map(|v| v.to_string());
    }
    if let Some(bin) = lower.strip_prefix("0b") {
        return u64::from_str_radix(bin, 2).ok().map(|v| v.to_string());
    }

    let (number, target) = lower
        .split_once(" in ")
        .or_else(|| lower.split_once(" to "))?;
    let value: u64 = number.trim().parse().ok()?;
    match target.trim() {
        "hex" => Some(format!("0x{value:X}")),
        "bin" | "binary" => Some(format!("0b{value:b}")),
        _ => None,
    }
}

/// `100 f to c`, `212f in c`, `2gb to mb`, `5 km in mi`, `3 lb to kg`.
fn convert_unit(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let (quantity, target) = lower
        .split_once(" to ")
        .or_else(|| lower.split_once(" in "))?;
    let quantity = quantity.trim();
    let split = quantity.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')?;
    let value: f64 = quantity[..split].trim().parse().ok()?;
    let from = quantity[split..].trim();
    let to = target.trim();

    if let Some(result) = temperature(value, from, to) {
        return Some(result);
    }
    for (units, decimals) in [(DATA, 2), (LENGTH, 4), (MASS, 4)] {
        let scale = |name: &str| {
            units
                .iter()
                .find(|(names, _)| names.contains(&name))
                .map(|(_, s)| *s)
        };
        if let (Some(a), Some(b)) = (scale(from), scale(to)) {
            let label = units
                .iter()
                .find(|(names, _)| names.contains(&to))
                .map(|(n, _)| n[0])?;
            let result = format!("{:.*}", decimals, value * a / b);
            let result = if result.contains('.') {
                result
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            } else {
                result
            };
            return Some(format!("{result} {label}"));
        }
    }
    None
}

type UnitTable = &'static [(&'static [&'static str], f64)];

const DATA: UnitTable = &[
    (&["B", "b", "byte", "bytes"], 1.0),
    (&["KB", "kb", "kib"], 1024.0),
    (&["MB", "mb", "mib"], 1024.0 * 1024.0),
    (&["GB", "gb", "gib"], 1024.0 * 1024.0 * 1024.0),
    (&["TB", "tb", "tib"], 1024.0 * 1024.0 * 1024.0 * 1024.0),
];

const LENGTH: UnitTable = &[
    (&["mm", "millimeters"], 0.001),
    (&["cm", "centimeters"], 0.01),
    (&["m", "meters"], 1.0),
    (&["km", "kilometers"], 1000.0),
    (&["in", "inch", "inches"], 0.0254),
    (&["ft", "feet", "foot"], 0.3048),
    (&["mi", "miles", "mile"], 1609.344),
];

const MASS: UnitTable = &[
    (&["g", "grams"], 1.0),
    (&["kg", "kilograms"], 1000.0),
    (&["oz", "ounces"], 28.349_523_125),
    (&["lb", "lbs", "pounds"], 453.592_37),
];

fn temperature(value: f64, from: &str, to: &str) -> Option<String> {
    let unit = |s: &str| match s {
        "c" | "celsius" => Some('C'),
        "f" | "fahrenheit" => Some('F'),
        "k" | "kelvin" => Some('K'),
        _ => None,
    };
    let celsius = match unit(from)? {
        'C' => value,
        'F' => (value - 32.0) * 5.0 / 9.0,
        _ => value - 273.15,
    };
    let (result, label) = match unit(to)? {
        'C' => (celsius, "°C"),
        'F' => (celsius * 9.0 / 5.0 + 32.0, "°F"),
        _ => (celsius + 273.15, "K"),
    };
    Some(format!(
        "{} {label}",
        format_number((result * 10.0).round() / 10.0)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(input: &str) -> Option<String> {
        evaluate(input)
    }

    #[test]
    fn arithmetic_respects_precedence() {
        assert_eq!(eval("2 + 3 * 4").as_deref(), Some("14"));
        assert_eq!(eval("(2 + 3) * 4").as_deref(), Some("20"));
        assert_eq!(eval("2 ^ 10").as_deref(), Some("1024"));
        assert_eq!(eval("2 ^ 3 ^ 2").as_deref(), Some("512"));
        assert_eq!(eval("10 / 4").as_deref(), Some("2.5"));
        assert_eq!(eval("3 - 10").as_deref(), Some("-7"));
        assert_eq!(eval("-(2 + 3)").as_deref(), Some("-5"));
        assert_eq!(eval("2 * -3").as_deref(), Some("-6"));
        assert_eq!(eval("1 / 3").as_deref(), Some("0.333333"));
    }

    #[test]
    fn invalid_or_non_finite_math_is_rejected() {
        assert_eq!(eval("1 / 0"), None);
        assert_eq!(eval("(-8) ^ 0.5"), None);
        assert_eq!(eval("10 ^ 400"), None);
        assert_eq!(eval("2 +"), None);
        assert_eq!(eval("my-file"), None);
    }

    #[test]
    fn plain_words_and_numbers_are_not_math() {
        assert_eq!(eval("hello"), None);
        assert_eq!(eval("42"), None);
    }

    #[test]
    fn base_conversion() {
        assert_eq!(eval("0xFF").as_deref(), Some("255"));
        assert_eq!(eval("0b1111").as_deref(), Some("15"));
        assert_eq!(eval("255 in hex").as_deref(), Some("0xFF"));
        assert_eq!(eval("5 to binary").as_deref(), Some("0b101"));
        assert_eq!(eval("0xZZ"), None);
    }

    #[test]
    fn temperature_conversion() {
        assert_eq!(eval("212f to c").as_deref(), Some("100 °C"));
        assert_eq!(eval("100 c in f").as_deref(), Some("212 °F"));
        assert_eq!(eval("0 c to k").as_deref(), Some("273.2 K"));
        assert_eq!(eval("-40 f to c").as_deref(), Some("-40 °C"));
    }

    #[test]
    fn unit_conversion() {
        assert_eq!(eval("2gb in mb").as_deref(), Some("2048 MB"));
        assert_eq!(eval("1 mi to km").as_deref(), Some("1.6093 km"));
        assert_eq!(eval("12 in to cm").as_deref(), Some("30.48 cm"));
        assert_eq!(eval("30 cm in in").as_deref(), Some("11.811 in"));
        assert_eq!(eval("1 kg to lb").as_deref(), Some("2.2046 lb"));
        assert_eq!(eval("5 kg to km"), None);
    }
}
