mod type_handling;
use type_handling::*;

use pest::{Parser, RuleType, iterators::Pair};
use pest_derive::Parser;

use crate::{
    ast::{
        self, Definition, Implementation, Import, TraitImplementation, Variable, Visibility,
        expressions::{
            Expression, Statement,
            block::Block,
            literal::Literal,
            loops::WhileLoop,
            operations::{BinaryOperation, BinaryOperator, UnaryOperator},
        },
        function::FunctionBuilder,
        module::{Module, ModuleKind},
    },
    general::{
        naming::{QualifiedName, QualifiedNameSegment},
        types::{PrimitiveType, Type},
    },
};

#[derive(Parser)]
#[grammar = "./pest/tokens.pest"]
pub struct TokenStream;

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
        un => unreachable!("{next:?}: {un:?}"),
    }

    return Ok(builder);
}

pub fn handle_member_access_postfix<'a>(
    pair: Pair<'a, Rule>,
) -> anyhow::Result<QualifiedNameSegment> {
    assert_eq!(
        pair.as_rule(),
        Rule::member_access_postfix,
        "a non Rule::member_access_postfix reached handle_member_access_postfix"
    );
    let mut inner = pair.into_inner();
    let member = inner.next().unwrap();
    return Ok(QualifiedNameSegment::from(member.as_str()));
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
    return Ok(handle_expression(inner.next().unwrap())?);
}

pub fn handle_call_expression_postfix<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<Expression>> {
    assert_eq!(
        pair.as_rule(),
        Rule::call_postfix,
        "a non Rule::call_postfix reached handle_call_expression_postfix"
    );
    let a = pair.into_inner().next();
    if a.is_none() {
        return Ok(vec![]);
    }
    let inner = a.unwrap().into_inner();
    let mut params = Vec::new();
    for innr in inner {
        params.push(handle_expression(innr)?);
    }

    return Ok(params);
}

pub fn handle_binary_expression_postfix<'a>(
    pair: Pair<'a, Rule>,
) -> anyhow::Result<(BinaryOperator, Expression)> {
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

    return Ok((operator, exp));
}

fn unescape_string_literal(raw: &str) -> String {
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }

    out
}

pub fn handle_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Expression> {
    assert_eq!(
        pair.as_rule(),
        Rule::expression,
        "a non Rule::expression reached handle_expression"
    );
    let mut inner = pair.clone().into_inner();
    let next = inner.next().unwrap();
    let prefix = match next.as_rule() {
        Rule::expression_priority => {
            let mut expr = next.into_inner();
            handle_expression(expr.next().unwrap())?
        }
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
                    let literal = Literal::string(unescape_string_literal(literal.as_str()));
                    Expression::literal(literal)
                }
                _ => unreachable!(""),
            }
        }
        Rule::path => Expression::identifier(handle_path(next)?),
        Rule::block_expression => Expression::block(handle_block_definition(next)?),
        Rule::while_expression => {
            let mut inner = next.into_inner();
            let condition = handle_expression(inner.next().unwrap())?;
            let block = Expression::block(handle_block_definition(inner.next().unwrap())?);

            Expression::while_(WhileLoop::new(condition, block))
        }
        Rule::prefix_unary_operation_expression => {
            let mut inner = next.into_inner();
            let oper = match inner.next().unwrap().as_str() {
                "&" => UnaryOperator::Ref,
                "*" => UnaryOperator::Deref,
                "-" => UnaryOperator::Negation,
                un => unreachable!("unreachable unary operator: {un:?}"),
            };

            return Ok(Expression::unary_operation(
                oper,
                handle_expression(inner.next().unwrap())?,
            ));
        }
        un => unreachable!("\"{}\": {un:?}", pair.as_str()),
    };

    let mut postfix_rules = Vec::new();

    while let Some(postfix) = inner.next() {
        assert_eq!(postfix.as_rule(), Rule::expression_postfix, "non postfix");
        let postfix = postfix.into_inner().next().unwrap();
        match postfix.as_rule() {
            Rule::call_postfix => {}
            Rule::assignment_expression_postfix => {}
            Rule::binary_expression_postfix => {}
            Rule::unary_expression_postfix => {}
            Rule::member_access_postfix => {}
            un => unreachable!("{un:?}"),
        }
        postfix_rules.push(postfix);
    }
    #[derive(Debug)]
    enum Operation {
        Call(Vec<Expression>),
        Assign(Expression),
        Access(QualifiedNameSegment),
        Unary(UnaryOperator),
        Binary(BinaryOperator),
    }

    impl PartialEq for Operation {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Operation::Call(_), Operation::Call(_)) => true,
                (Operation::Access(a), Operation::Access(b)) => a == b,
                (Operation::Unary(a), Operation::Unary(b)) => a == b,
                (Operation::Binary(a), Operation::Binary(b)) => a == b,
                _ => false,
            }
        }
    }

    impl PartialOrd for Operation {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(match (self, other) {
                (Operation::Assign(_), _) => std::cmp::Ordering::Equal,
                (Operation::Call(_), Operation::Binary(_)) => std::cmp::Ordering::Greater,
                (Operation::Call(_), _) => std::cmp::Ordering::Equal,

                (Operation::Access(_), Operation::Binary(_)) => std::cmp::Ordering::Greater,
                (Operation::Access(_), _) => std::cmp::Ordering::Equal,

                (Operation::Unary(a), Operation::Unary(b)) => a.cmp(b),
                (Operation::Unary(_), Operation::Binary(_)) => std::cmp::Ordering::Greater,
                (Operation::Unary(_), _) => std::cmp::Ordering::Equal,

                (Operation::Binary(a), Operation::Binary(b)) => a.cmp(b),
                (Operation::Binary(_), _) => std::cmp::Ordering::Less,
            })
        }
    }

    #[derive(Debug)]
    enum ExpressionString {
        Val(Expression),
        Oper(Operation),
    }

    impl ExpressionString {
        fn to_val(self) -> anyhow::Result<Expression> {
            if let ExpressionString::Val(p) = self {
                return Ok(p);
            } else {
                anyhow::bail!("ExpressionString was not val it was: {self:?}")
            }
        }
    }

    let mut e_string = Vec::new();
    e_string.push(ExpressionString::Val(prefix));
    for p in postfix_rules {
        match p.as_rule() {
            Rule::binary_expression_postfix => {
                let (a, b) = handle_binary_expression_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Binary(a)));
                e_string.push(ExpressionString::Val(b));
            }
            Rule::unary_expression_postfix => {
                todo!()
            }
            Rule::call_postfix => {
                let params = handle_call_expression_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Call(params)));
            }
            Rule::member_access_postfix => {
                let ident = handle_member_access_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Access(ident)));
            }
            Rule::assignment_expression_postfix => {
                let mut ex = p.into_inner();
                e_string.push(ExpressionString::Oper(Operation::Assign(
                    handle_expression(ex.next().unwrap())?,
                )));
            }
            un => todo!("{un:?}:'{}'", p.as_str()),
        }
    }

    let mut out_q: Vec<ExpressionString> = Vec::new();
    let mut op_st = Vec::new();

    for e in e_string {
        match e {
            ExpressionString::Val(v) => {
                out_q.push(ExpressionString::Val(v));
            }
            ExpressionString::Oper(o) => {
                if op_st.len() != 0 {
                    while let Some(lst) = op_st.last() {
                        if *lst >= o {
                            let lst = op_st.pop().unwrap();
                            out_q.push(ExpressionString::Oper(lst));
                        } else {
                            break;
                        }
                    }
                    op_st.push(o);
                } else {
                    op_st.push(o);
                }
            }
        }
    }
    for lst in op_st.into_iter().rev() {
        out_q.push(ExpressionString::Oper(lst));
    }

    fn handle_collapse(
        o: Operation,
        out_q: &mut Vec<ExpressionString>,
    ) -> anyhow::Result<Expression> {
        let mut get_next_val = || -> anyhow::Result<Expression> {
            match out_q.pop().unwrap() {
                ExpressionString::Val(v) => Ok(v),
                ExpressionString::Oper(o) => handle_collapse(o, out_q),
            }
        };
        match o {
            Operation::Access(ident) => {
                let a = get_next_val()?;

                return Ok(Expression::member_access(a, ident));
            }
            Operation::Call(params) => {
                let a = get_next_val()?;

                return Ok(Expression::call(a, params));
            }
            Operation::Unary(o) => {
                let a = get_next_val()?;
                return Ok(Expression::unary_operation(o, a));
            }
            Operation::Binary(o) => {
                let b = get_next_val()?;
                let a = get_next_val()?;

                return Ok(Expression::binary_operation(o, a, b));
            }
            Operation::Assign(b) => {
                let a = get_next_val()?;
                return Ok(Expression::assignment(a, b));
            }
        }
    }

    while out_q.len() > 1 {
        if let ExpressionString::Oper(o) = out_q.pop().unwrap() {
            let e = handle_collapse(o, &mut out_q)?;
            out_q.push(ExpressionString::Val(e));
        }
    }

    return out_q.pop().unwrap().to_val();
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
    let ty = if let Rule::type_expr = inner.peek().unwrap().as_rule() {
        Some(handle_type_expression(inner.next().unwrap())?)
    } else {
        None
    };
    let expr = handle_expression(inner.next().unwrap())?;

    let var = Definition::variable(ident, expr, mutable, ty)?;

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
    assert_eq!(
        pair.as_rule(),
        Rule::block_expression,
        "a non Rule::block_expression reached handle_definitions"
    );
    let inner = pair.into_inner();
    let mut block = Block::new();
    for stmt in inner {
        block.add_statement(handle_statement(stmt)?);
    }

    return Ok(block);
}

pub fn handle_fn_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::fn_definition,
        "a non Rule::definition reached handle_definitions"
    );
    let mut inner = pair.into_inner();
    let declaration = inner.next().unwrap();
    let block = inner.next().unwrap();
    let function = handle_fn_declaration(declaration)?.set_body(handle_block_definition(block)?);

    return Ok(function.build_def());
}

pub fn handle_trait_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    todo!("implement into the trait definition ast")
}

pub fn handle_path_ident<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<QualifiedNameSegment> {
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str().to_string();
    if let Some(tm) = inner.next() {
        todo!("handle template specialization");
    }

    return Ok(QualifiedNameSegment {
        ident,
        template_spec: vec![],
    });
}

pub fn handle_path<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<QualifiedName> {
    let mut inner = pair.into_inner();
    let next = inner.peek().unwrap();
    let mut path = QualifiedName::new();

    if let Rule::rel_path = next.as_rule() {
        todo!("handle relative paths...");
    }

    let mut path = QualifiedName::new();
    for ident in inner {
        path.add_segment(handle_path_ident(ident)?);
    }

    return Ok(path);
}

pub fn handle_implementation<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Implementation> {
    assert_eq!(pair.as_rule(), Rule::implementation);
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

pub fn handle_trait_implementation<'a>(
    pair: Pair<'a, Rule>,
) -> anyhow::Result<TraitImplementation> {
    assert_eq!(pair.as_rule(), Rule::trait_implementation);
    let mut inner = pair.into_inner();
    let target = handle_path(inner.next().unwrap())?;
    let trait_path = handle_path(inner.next().unwrap())?;
    let mut definitions = Vec::new();

    while let Some(ok) = inner.next() {
        definitions.push(handle_fn_definitions(ok)?);
    }

    Ok(TraitImplementation {
        trait_path,
        target,
        definitions,
    })
}
pub fn handle_module_definition<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(pair.as_rule(), Rule::module_definition);
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap();

    if inner.next().is_some() {
        todo!("handle infine module");
    }

    let mut md = Module::new();
    md.kind = ModuleKind::ExFile;

    return Ok(Definition::module(ident.as_str(), md));
}

pub fn handle_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::definitions,
        "a non Rule::definition reached handle_definitions"
    );
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

pub fn parse_module<S: AsRef<str>>(txt: S) -> anyhow::Result<Module> {
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
                match def.as_rule() {
                    Rule::implementation => md.impls.push(handle_implementation(def)?),
                    Rule::trait_implementation => {
                        md.trait_impls.push(handle_trait_implementation(def)?)
                    }
                    un => unreachable!("{un:?}"),
                }
            }
            un => unreachable!("{un:?}"),
        }
    }

    return Ok(md);
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_operator_precedence() -> anyhow::Result<()> {
        let k = TokenStream::parse(Rule::expression, "a().b + c.d()")?
            .next()
            .unwrap();
        assert_eq!(k.as_rule(), Rule::expression);
        let parsed = handle_expression(k)?;

        let expected = Expression::binary_operation(
            BinaryOperator::Addition,
            Expression::member_access(Expression::call(Expression::identifier("a"), vec![]), "b"),
            Expression::call(
                Expression::member_access(Expression::identifier("c"), "d"),
                vec![],
            ),
        );

        assert!(parsed == expected, "{parsed} != {expected}");

        return Ok(());
    }

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
            "func main() -> i32 {
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
            "inter TestTrait<T: A<T>>{\n\ttype DeclaredType = T;\n\ttype DefinedType = T;\n\tfunc func() -> Type;\n}",
        )?;

        TokenStream::parse(
            Rule::trait_definition,
            "inter testTrait<T: Add<T>>{
    type TestType = i32;
    
    func test_function<T: Add<T>>(t: T) -> T ;
}",
        )?;

        return Ok(());
    }

    #[test]
    fn test_extend_impl() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::trait_implementation,
            "extend Vec2 as Add<Vec2> {
    func add(other: Vec2) -> Vec2 {
        return self;
    }
}",
        )?;

        Ok(())
    }
    #[test]
    fn test_idents() -> anyhow::Result<()> {
        TokenStream::parse(Rule::stream, "")?;
        TokenStream::parse(Rule::ident_root, "test_ident<T: A<T>, U: B>")?;
        TokenStream::parse(Rule::ident_root, "test_ident")?;

        TokenStream::parse(
            Rule::fn_declaration,
            "func test_fn_declaration(t: T, u: U) -> U",
        )?;
        TokenStream::parse(
            Rule::fn_declaration,
            "func test_fn_declaration_template<T: global::A<T> >(t: T, u: U) -> U",
        )?;
        return Ok(());
    }

    #[test]
    fn test_call_expression() -> anyhow::Result<()> {
        TokenStream::parse(Rule::expression, "printf(\"%d\", i)")?;

        Ok(())
    }

    #[test]
    fn test_string_escape_newline() -> anyhow::Result<()> {
        let expr = TokenStream::parse(Rule::expression, "\"a\\nb\"")?
            .next()
            .unwrap();
        let parsed = handle_expression(expr)?;

        let expected = Expression::literal(Literal::string("a\nb"));
        assert_eq!(parsed, expected);

        Ok(())
    }

    #[test]
    fn test_min_clike() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::module,
            "header stdio::printf;

type TestType = i32;

func main() -> i32 {
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
