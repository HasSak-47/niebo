use pest::{Parser, RuleType, iterators::Pair};
use pest_derive::Parser;

use crate::ast::{self, Import, Module, Path};

#[derive(Parser)]
#[grammar = "./pest/tokens.pest"]
struct TokenStream;

#[derive(Debug, Default)]
struct Identifier {
    path: Vec<String>,
}

#[derive(Debug, Default)]
enum TokenKind {
    Identifier(Identifier),
    Number,
    #[default]
    Symbol,
}

#[derive(Debug, Default)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,

    line: usize,
    col: usize,
}

pub fn print_token<'a, T: RuleType>(t: Pair<'a, T>, depth: usize, max: usize) {
    println!("{:\t>depth$}'{}':{:?}", "", t.as_str(), t.as_rule());
    if depth > max {
        return;
    }
    for t in t.into_inner() {
        print_token(t, depth + 1, max);
    }
}

pub fn parse_module<S: AsRef<str>>(txt: S) -> anyhow::Result<ast::Module> {
    let md = Module::new();
    let ts = TokenStream::parse(Rule::module, txt.as_ref())?;
    for t in ts.into_iter().next().unwrap().into_inner() {
        print_token(t, 0, 1);
    }

    return Ok(md);
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_functions() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::fn_definition,
            "fn main() -> i32 {
    let i = 0;
    while i < 10 {
        i++;
    }
    return 0;
}",
        )?;
        TokenStream::parse(Rule::ident_root, "a: A<T>, b: B, c: B<T: S<T>>")?;

        return Ok(());
    }

    #[test]
    fn test_params() -> anyhow::Result<()> {
        TokenStream::parse(Rule::ident_root, "a: A, b: B, c: B")?;
        TokenStream::parse(Rule::ident_root, "a: A<T>, b: B, c: B<T: S<T>>")?;

        return Ok(());
    }

    #[test]
    fn test_traits() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::definitions,
            "trait TestTrait<T: A<T>>{\n\ttype DeclaredType = T;\n\ttype DefinedType = T;\n\tfn func() -> Type;\n}",
        )?;

        TokenStream::parse(
            Rule::trait_definition,
            "trait testTrait<T: Add<T>>{
    type TestType = int32;
    
    fn test_function<T: Add<T>>(t: T) -> T ;
}",
        )?;

        return Ok(());
    }
    #[test]
    fn test_idents() -> anyhow::Result<()> {
        TokenStream::parse(Rule::stream, "")?;
        TokenStream::parse(Rule::ident_root, "test_ident<T: A<T>, U: B>")?;
        TokenStream::parse(Rule::ident_root, "test_ident")?;

        TokenStream::parse(
            Rule::fn_declaration,
            "fn test_fn_declaration(t: T, u: U) -> U",
        )?;
        TokenStream::parse(
            Rule::fn_declaration,
            "fn test_fn_declaration_template<T: global::A<T> >(t: T, u: U) -> U",
        )?;
        return Ok(());
    }
}
