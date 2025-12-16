pub mod lowlevel;
fn remove_comments(mut code: Vec<String>) -> Vec<String> {
    let mut inside_comment = false;
    for line in &mut code {
        if inside_comment {
            if let Some(indx) = line.find("*/") {
                *line = line.split_off(indx);
                inside_comment = false;
            } else {
                continue;
            }
        }
        if let Some(indx) = line.find("/*") {
            line.truncate(indx);
            inside_comment = true;
            continue;
        }
        if let Some(indx) = line.find("//") {
            line.truncate(indx);
        }
    }

    return code;
}

pub enum LiteralToken {
    String,
    Char,
    Number,
}

pub enum TokenType {
    Literal(LiteralToken),
    Identifier(String),
    Punctuation(String),
}

pub struct Token {
    token: TokenType,
    pos: (usize, usize),
    val: String,
}

const SYMBOLS: &[&str] = &[
    "...", "..", // ranges
    "->", "=>", // arrows
    "&&", "||", // boolean and or
    ">>", "<<", // bit shift
    ">=", "<=", "==", // comparison
    "{", "}", "<", ">", "(", ")", "[", "]", // brackets
    "~", "!", "?", // error stuff and negation ig
    "&", "|", "^", "~", // bit manipulation
    "+", "-", "*", "/", "%", // algebra
    ".", ",", ":", ";", "\"", "'", // delimitators and others
];

pub fn split_simbols<S: AsRef<str>>(code: S) {
    let mut current_str = code.as_ref();
    let mut chunks = Vec::new();
    loop {
        if current_str.is_empty() {
            break;
        }
        println!("{current_str:?}");

        let mut splits = Vec::new();
        for symbol in SYMBOLS {
            if let Some(at) = current_str.find(symbol) {
                splits.push((at, symbol.len()));
            }
        }

        splits.sort_by(|a, b| a.0.cmp(&b.0));
        let (prev, after) = current_str.split_at(splits[0].0 + splits[0].1);
        let (prev, symbol) = prev.split_at(splits[0].0);
        if !prev.is_empty() {
            chunks.push(prev);
        }
        chunks.push(symbol);
        current_str = after;
    }
    println!("{chunks:?}");
}

pub fn clean_up(code: String) {
    let mut code = remove_comments(code.lines().map(str::to_string).collect()).join(" ");
    code = code.replace("\t", " ");
    code = code.replace("\n", " ");
    while code.contains("  ") {
        code = code.replace("  ", " ");
    }
}
