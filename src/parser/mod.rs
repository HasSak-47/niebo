use pest::{Parser, RuleType, iterators::Pair};
use pest_derive::Parser;

use crate::ast::{
    self, Definition, Import, Module, Path, expressions::block::Block, function::FunctionBuilder,
};

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

pub fn handle_fn_declaration<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<FunctionBuilder> {
    todo!("implement into the fn declaration ast")
}

pub fn handle_block_definition<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Block> {
    todo!("implement into the block definition ast")
}

pub fn handle_fn_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    let mut inner = pair.into_inner();
    let declaration = inner.next().unwrap();
    let block = inner.next().unwrap();
    let function = handle_fn_declaration(declaration)?.set_body(handle_block_definition(block)?);

    return Ok(function.build_def());
}

pub fn handle_trait_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    todo!("implement into the trait definition ast")
}

pub fn handle_type_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().into_inner();
    let def = inner.next().unwrap().into_inner();

    let type_ident = ident.as_str();
    let type_def = def.as_str();

    return Ok(Definition::type_def(
        type_ident,
        Path::from(type_def),
        ast::Visibility::Public,
    ));
}

pub fn handle_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    // TODO: handle visibility
    let inner = pair
        .into_inner()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    match inner.as_rule() {
        Rule::type_definition => return handle_type_definitions(inner),
        Rule::fn_definition => return handle_fn_definitions(inner),
        Rule::trait_definition => return handle_trait_definitions(inner),
        un => unreachable!("{un:?}"),
    };

    todo!()
}

pub fn parse_module<S: AsRef<str>>(txt: S) -> anyhow::Result<ast::Module> {
    let mut md = Module::new();
    let ts = TokenStream::parse(Rule::module, txt.as_ref())?
        .into_iter()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap()
        .into_inner();
    for t in ts {
        match t.as_rule() {
            Rule::definitions => {
                md.definitions.push(handle_definitions(t)?);
            }
            Rule::import => {}
            un => unreachable!("{t:?}"),
        }
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
