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
    Literal {
        literal: LiteralToken,
        specifier: String,
    },
    Identifier(String),
    Punctuation(String),
}

pub struct Token {
    token: TokenType,
    pos: (usize, usize),
    val: String,
}

pub fn parse(code: String) {
    let mut code = remove_comments(code.lines().map(str::to_string).collect()).join("");
    code = code.replace("\t", " ");
    code = code.replace("\n", " ");
    while code.contains("  ") {
        code = code.replace("  ", " ");
    }

    let vals = code.split(" ");
}
