use super::{Rule, handle_qualified_name, handle_statement};

use pest::iterators::Pair;

use crate::{
    ast::expressions::{
        Expression, ExpressionKind,
        block::Block,
        conditional::{Conditional, ConditionalBuilder},
        init::StructInit,
        literal::Literal,
        loops::{LoopExpression, WhileLoop},
        operations::{BinaryOperator, UnaryOperator},
    },
    general::naming::QualifiedNameSegment,
};

fn handle_index_access_postfix<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Expression> {
    assert_eq!(
        pair.as_rule(),
        Rule::index_access_postfix,
        "a non Rule::index_access_postfix reached handle_index_access_postfix"
    );
    let mut inner = pair.into_inner();
    let member = inner.next().unwrap();
    return Ok(handle_expression(member)?);
}

fn handle_member_access_postfix<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<QualifiedNameSegment> {
    assert_eq!(
        pair.as_rule(),
        Rule::member_access_postfix,
        "a non Rule::member_access_postfix reached handle_member_access_postfix"
    );
    let mut inner = pair.into_inner();
    let member = inner.next().unwrap();
    return Ok(QualifiedNameSegment::from(member.as_str()));
}

fn handle_assignment_expression_postfix<'a>(
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

fn handle_call_expression_postfix<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<Expression>> {
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

fn handle_unary_expression_postfix<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<UnaryOperator> {
    assert_eq!(
        pair.as_rule(),
        Rule::unary_expression_postfix,
        "a non Rule::unary_expression_postfix reached handle_unary_expression_postfix"
    );

    let mut inner = pair.into_inner();
    let oper = inner.next().unwrap();
    let operator = match &oper.as_rule() {
        Rule::unary_return_error_postfix => UnaryOperator::EarlyRet,
        Rule::unary_increase_postfix => UnaryOperator::Increase,
        Rule::unary_decrease_postfix => UnaryOperator::Decrease,
        un => unreachable!("{un:?}"),
    };

    return Ok(operator);
}

fn handle_binary_expression_postfix<'a>(
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
        Rule::boolean_eq => BinaryOperator::Equal,
        Rule::arithmetic_add => BinaryOperator::Addition,
        Rule::arithmetic_mul => BinaryOperator::Multiplication,
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

pub fn handle_block_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Block> {
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

fn handle_if_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Conditional> {
    assert_eq!(
        pair.as_rule(),
        Rule::if_expression,
        "a non Rule::if_expression reached handle_if_expression"
    );
    let mut inner = pair.into_inner();
    let cond = inner.next().unwrap();
    let then = inner.next().unwrap();
    let mut cond = ConditionalBuilder::new(
        handle_expression(cond)?,
        Expression::block(handle_block_expression(then)?),
    );
    if let Some(else_) = inner.next() {
        cond = cond.set_else(Expression::block(handle_block_expression(else_)?));
    }

    return Ok(cond.build());
}

fn handle_loop_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<LoopExpression> {
    assert_eq!(
        pair.as_rule(),
        Rule::loop_expression,
        "a non Rule::loop_expression reached handle_loop_expression"
    );
    let mut inner = pair.into_inner();
    let next = inner.next().unwrap();
    let mut ident = String::new();
    let body = match next.as_rule() {
        Rule::identifier => {
            ident = next.as_str().to_string();
            inner.next().unwrap()
        }
        Rule::expression => next,
        u => unreachable!("{u:?}"),
    };
    let mut exp = LoopExpression::new(Expression::block(handle_block_expression(body)?));

    if !ident.is_empty() {
        exp.label = Some(ident);
    }
    return Ok(exp);
}

#[derive(Debug)]
enum Operation {
    Call(Vec<Expression>),
    Assign(Expression),
    Access(QualifiedNameSegment),
    Index(Expression),
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
            (Operation::Index(_), _) => std::cmp::Ordering::Equal,

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
        Rule::path => Expression::identifier(handle_qualified_name(next)?),
        Rule::block_expression => Expression::block(handle_block_expression(next)?),
        Rule::loop_expression => Expression::loop_(handle_loop_expression(next)?),
        Rule::if_expression => Expression::if_(handle_if_expression(next)?),
        Rule::while_expression => {
            let mut inner = next.into_inner();
            let condition = handle_expression(inner.next().unwrap())?;
            let block = Expression::block(handle_block_expression(inner.next().unwrap())?);

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
        Rule::struct_expression => {
            let mut s = StructInit::default();
            let mut inner = next.into_inner();
            s.ident = handle_qualified_name(inner.next().unwrap())?;

            for i in inner {
                let mut named_field = i.into_inner();
                let ident = named_field.next().unwrap().as_str().to_string();
                let exp = handle_expression(named_field.next().unwrap())?;

                s.params.push((ident, exp));
            }

            return Ok(Expression::new(ExpressionKind::StructInit(s)));
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
            Rule::index_access_postfix => {}
            un => unreachable!("{un:?}"),
        }
        postfix_rules.push(postfix);
    }

    let mut e_string = Vec::new();
    e_string.push(ExpressionString::Val(prefix));
    for p in postfix_rules {
        match p.as_rule() {
            Rule::binary_expression_postfix => {
                let (operator, postfix) = handle_binary_expression_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Binary(operator)));
                e_string.push(ExpressionString::Val(postfix));
            }
            Rule::unary_expression_postfix => {
                let operator = handle_unary_expression_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Unary(operator)))
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
            Rule::index_access_postfix => {
                let ex = handle_index_access_postfix(p)?;
                e_string.push(ExpressionString::Oper(Operation::Index(ex)));
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
            Operation::Index(index) => {
                let a = get_next_val()?;
                return Ok(Expression::index_access(a, index));
            }
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
