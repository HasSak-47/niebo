use pest::{Parser, RuleType, iterators::Pair};
use pest_derive::Parser;

use crate::{
    ast::{
        self, Definition, Implementation, Import, Module, Variable, Visibility,
        expressions::{
            Expression, Statement,
            block::Block,
            literal::Literal,
            operations::{BinaryOperation, BinaryOperator},
        },
        function::FunctionBuilder,
    },
    general::{
        path::{Path, PathIdent},
        types::{PrimitiveType, Type},
    },
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

pub fn handle_fn_param<'a>(
    pair: Pair<'a, Rule>,
    mut builder: FunctionBuilder,
) -> anyhow::Result<FunctionBuilder> {
    assert_eq!(pair.as_rule(), Rule::param);
    let mut inner = pair.into_inner();

    let id_p = inner.next().unwrap();
    assert_eq!(id_p.as_rule(), Rule::identifier);

    let pt_p = inner.next().unwrap();
    assert_eq!(pt_p.as_rule(), Rule::type_expr);

    return Ok(builder.add_param(id_p.as_str(), handle_type_expression(pt_p)?));
}

pub fn handle_fn_params<'a>(
    pair: Pair<'a, Rule>,
    mut builder: FunctionBuilder,
) -> anyhow::Result<FunctionBuilder> {
    assert_eq!(pair.as_rule(), Rule::params);
    let mut inner = pair.into_inner();

    for param in inner {
        builder = handle_fn_param(param, builder)?;
    }

    return Ok(builder);
}

pub fn handle_template_def<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<()> {
    todo!()
}

pub fn handle_fn_declaration<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<FunctionBuilder> {
    let mut inner = pair.into_inner().into_iter();
    let ident = inner.next().unwrap();
    let mut builder = FunctionBuilder::new(ident.as_str());
    let next = inner.next().unwrap();
    match next.as_rule() {
        Rule::template_def => {
            handle_template_def(next);
            builder = handle_fn_params(inner.next().unwrap(), builder)?;
        }
        Rule::params => {
            builder = handle_fn_params(next, builder)?;
            if let Some(s) = inner.next() {
                builder = builder.set_ret_ty(handle_type_expression(s)?);
            }
        }

        // return type
        Rule::type_expr => {
            builder = builder.set_ret_ty(handle_type_expression(next)?);
        }
        un => unreachable!("{next:?}"),
    }

    return Ok(builder);
}

pub fn handle_member_access_postfix<'a>(
    prefix: Expression,
    pair: Pair<'a, Rule>,
) -> anyhow::Result<Expression> {
    todo!()
}

pub fn handle_assignment_expression_postfix<'a>(
    prefix: Expression,
    pair: Pair<'a, Rule>,
) -> anyhow::Result<Expression> {
    assert_eq!(
        pair.as_rule(),
        Rule::assignment_expression_postfix,
        "a non Rule::assignment_expression_postfix reached handle_assigment_operation_postfix"
    );
    let mut inner = pair.into_inner();
    return Ok(Expression::assignment(
        prefix,
        handle_expression(inner.next().unwrap())?,
    ));
}

pub fn handle_call_expression_postfix<'a>(
    prefix: Expression,
    pair: Pair<'a, Rule>,
) -> anyhow::Result<Expression> {
    assert_eq!(
        pair.as_rule(),
        Rule::call_postfix,
        "a non Rule::call_postfix reached handle_call_expression_postfix"
    );
    let inner = pair.into_inner().next().unwrap().into_inner();
    let mut params = Vec::new();
    for innr in inner {
        params.push(handle_expression(innr)?);
    }

    return Ok(Expression::call(prefix, params));
}

pub fn handle_binary_expression_postfix<'a>(
    prefix: Expression,
    pair: Pair<'a, Rule>,
) -> anyhow::Result<Expression> {
    assert_eq!(
        pair.as_rule(),
        Rule::binary_expression_postfix,
        "a non Rule::binary_expression_postfix reached handle_binary_expression_postfix"
    );

    let mut inner = pair.into_inner();
    let oper = inner.next().unwrap();
    let operator = match &oper.as_rule() {
        Rule::boolean_leq => BinaryOperator::LesserOrEqual,
        Rule::boolean_le => BinaryOperator::Lesser,
        Rule::arithmetic_add => BinaryOperator::Addition,
        un => unreachable!("{un:?}"),
    };
    let exp = handle_expression(oper.into_inner().next().unwrap())?;

    return Ok(Expression::binary_operation(operator, prefix, exp));
}

pub fn handle_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Expression> {
    let mut inner = pair.into_inner();
    let next = inner.next().unwrap();
    let prefix = match next.as_rule() {
        Rule::return_expression => {
            let mut expr = next.into_inner();
            let value = expr.next().and_then(|k| handle_expression(k).ok());
            Expression::return_(value)
        }
        Rule::literal_expression => {
            let literal = next.into_inner().next().unwrap();
            match literal.as_rule() {
                Rule::number => {
                    let literal = Literal::integer(literal.as_str(), None, None);
                    Expression::literal(literal)
                }
                Rule::string => {
                    let literal = Literal::string(literal.as_str());
                    Expression::literal(literal)
                }
                _ => unreachable!(""),
            }
        }
        Rule::path => Expression::identifier(handle_path(next)?),
        Rule::block_expression => Expression::block(handle_block_definition(next)?),
        un => unreachable!("{un:?}"),
    };
    let postfix = inner.next();
    if let None = postfix {
        return Ok(prefix);
    }

    // TODO: create a postfix RPN generator because that is the most normal way to do it instead of
    // left to right
    let postfix = postfix.unwrap().into_inner().next().unwrap();
    return Ok(match postfix.as_rule() {
        Rule::binary_expression_postfix => handle_binary_expression_postfix(prefix, postfix)?,
        Rule::call_postfix => handle_call_expression_postfix(prefix, postfix)?,
        Rule::assignment_expression_postfix => {
            handle_assignment_expression_postfix(prefix, postfix)?
        }
        Rule::call_postfix | Rule::postfix_unary_operator => {
            todo!("{:?}", postfix.as_rule())
        }
        Rule::member_access_postfix => handle_member_access_postfix(prefix, postfix)?,
        un => unreachable!("{un:?}"),
    });
}

pub fn handle_let_declaration<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Statement> {
    assert_eq!(
        pair.as_rule(),
        Rule::let_declaration,
        "non let_declaration reached handle_let_declaration!"
    );

    let mut inner = pair.into_inner();

    let mutable = if let Rule::mutable_modifier = inner.peek().unwrap().as_rule() {
        inner.next();
        true
    } else {
        false
    };

    let ident = inner.next().unwrap().as_str();
    let expr = handle_expression(inner.next().unwrap())?;

    let var = Definition::variable::<&str, Path>(ident, expr, mutable, None)?;

    return Ok(Statement::Definition(var));
}

pub fn handle_statement<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Statement> {
    assert_eq!(
        pair.as_rule(),
        Rule::statement,
        "non statement reached handle_statement!"
    );
    let mut inner = pair.into_inner();

    let next = inner.next().unwrap();
    return Ok(match next.as_rule() {
        Rule::expression => Statement::Expression(handle_expression(next)?),
        Rule::let_declaration => handle_let_declaration(next)?,
        Rule::import
        | Rule::const_definition
        | Rule::break_statement
        | Rule::continue_statement => todo!(),
        un => unreachable!("{un:?}"),
    });
}

pub fn handle_block_definition<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Block> {
    let inner = pair.into_inner();
    let mut block = Block::new();
    for stmt in inner {
        block.add_statement(handle_statement(stmt)?);
    }

    return Ok(block);
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

pub fn handle_primitive_type<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<PrimitiveType> {
    let mut inner = pair.into_inner();
    let next = inner.next().unwrap();
    match next.as_rule() {
        Rule::int_type => return Ok(PrimitiveType::Int(0)),
        un => unreachable!(""),
    }
    todo!("{:?}", next.as_rule());
}

pub fn handle_type_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Type> {
    let mut inner = pair.into_inner();
    let ty = inner.next().unwrap();
    return match ty.as_rule() {
        Rule::path => Ok(Type::named(handle_path(ty)?)),
        Rule::primitive_type => Ok(Type::Primitive(handle_primitive_type(ty)?)),
        Rule::mutable_reference_type => {
            let mut inner = ty.into_inner();
            let next = inner.next().unwrap();
            Ok(Type::MutableReference(Box::new(handle_type_expression(
                next,
            )?)))
        }
        un => unreachable!("{un:?}"),
    };
}

pub fn handle_path_ident<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<PathIdent> {
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str().to_string();
    if let Some(tm) = inner.next() {
        todo!("handle template specialization");
    }

    return Ok(PathIdent {
        ident,
        template_spec: vec![],
    });
}

pub fn handle_path<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Path> {
    let mut inner = pair.into_inner();
    let next = inner.peek().unwrap();
    let mut path = Path::new();

    if let Rule::rel_path = next.as_rule() {
        todo!("handle relative paths...");
    }

    let mut path = Path::new();
    for ident in inner {
        path.add_segment(handle_path_ident(ident)?);
    }

    return Ok(path);
}

pub fn handle_implementation<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Implementation> {
    assert_eq!(pair.as_rule(), Rule::implementation);
    println!("{pair:?}");
    let mut inner = pair.into_inner();
    let target = handle_path(inner.next().unwrap())?;
    let mut definitions = Vec::new();

    while let Some(ok) = inner.next() {
        definitions.push(handle_fn_definitions(ok)?);
    }

    return Ok(Implementation {
        target,
        definitions,
    });
}
pub fn handle_module_definition<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(pair.as_rule(), Rule::module_definition);
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap();

    if inner.next().is_some() {
        todo!("handle infine module");
    }

    let mut md = Module::new();
    md.kind = ast::ModuleKind::ExFile;

    return Ok(Definition::module(ident.as_str(), md));
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
    let mut inner = pair.into_inner().next().unwrap().into_inner();

    let vis = if let Rule::visibility = inner.peek().unwrap().as_rule() {
        // TODO: handle global public vs module public
        inner.next();
        Visibility::Private
    } else {
        Visibility::Private
    };

    let next = inner.next().unwrap();
    return match next.as_rule() {
        Rule::type_definition => handle_type_definitions(next),
        Rule::fn_definition => handle_fn_definitions(next),
        Rule::trait_definition => handle_trait_definitions(next),
        Rule::module_definition => handle_module_definition(next),
        un => unreachable!("{un:?}"),
    }
    .and_then(|mut k| {
        k.visibility = vis;
        Ok(k)
    });
}

pub fn handle_c_imports<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Import> {
    let inner = pair.into_inner().next().unwrap();
    assert_eq!(inner.as_rule(), Rule::path);

    return Ok(Import::c_import(handle_path(inner)?));
}

pub fn handle_imports<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Import> {
    let inner = pair.into_inner().next().unwrap();
    return match inner.as_rule() {
        Rule::c_import => handle_c_imports(inner),
        Rule::niebo_import => todo!("handle niebo imports..."),
        _ => unreachable!(),
    };
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
            Rule::import => {
                md.imports.push(handle_imports(t)?);
            }
            Rule::impls => {
                let def = t.into_inner().next().unwrap();
                md.impls.push(handle_implementation(def)?);
            }
            un => unreachable!("{t:?}"),
        }
    }

    return Ok(md);
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_member_access() -> anyhow::Result<()> {
        //  TODO: check that the postfix is correct
        // expression postfix:call postfix:access postfix:call
        let mut access = TokenStream::parse(Rule::expression, "a().b(10)")?;
        let exp = access.next().unwrap();
        assert_eq!(exp.as_rule(), Rule::expression);
        for a in access {
            assert_eq!(a.as_rule(), Rule::expression_postfix);
        }

        return Ok(());
    }

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

    #[test]
    fn test_call_expression() -> anyhow::Result<()> {
        TokenStream::parse(Rule::expression, "printf(\"%d\", i)")?;

        Ok(())
    }

    #[test]
    fn test_min_clike() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::module,
            "header stdio::printf;

type TestType = int32;

fn main() -> i32 {
    let i = 0;
    while i < 10 {
        printf(\"%d\", i);
        i = i + 1;
    }
    return 0;
} ",
        )?;

        return Ok(());
    }
}
