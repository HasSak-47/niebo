use pest::{Parser, RuleType, iterators::Pair};
use pest_derive::Parser;

use crate::{
    ast::{
        self, Definition, Implementation, Import, Variable, Visibility,
        expressions::{
            Expression, Statement,
            block::Block,
            literal::Literal,
            operations::{BinaryOperation, BinaryOperator, UnaryOperator},
        },
        function::FunctionBuilder,
        module::{Module, ModuleKind},
    },
    general::{
        path::{Path, PathIdent},
        types::{PrimitiveType, Type},
    },
    parser::*,
};

fn handle_primitive_type<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<PrimitiveType> {
    assert_eq!(
        pair.as_rule(),
        Rule::primitive_type,
        "handle_primitive_type got a non primitive_type"
    );
    let mut inner = pair.into_inner();
    let next = inner.next().unwrap();
    match next.as_rule() {
        Rule::int_type => return Ok(PrimitiveType::Int(0)),
        un => unreachable!("{un:?}"),
    }
}

pub fn handle_type_alias_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::alias_definition,
        "handle_type_alias_definitions got a non alias_definition"
    );
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().to_string();
    let path = inner.next().unwrap();

    return Ok(Definition::type_def(
        ident,
        Type::Alias(Box::new(Type::named(handle_path(path)?))),
        Visibility::Public,
    ));
}

pub fn handle_type_struct_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::struct_definition,
        "handle_type_struct_definitions got a non struct_definition"
    );
    todo!()
}

pub fn handle_type_union_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::union_definition,
        "handle_type_union_definitions got a non union_definition"
    );
    todo!()
}

pub fn handle_type_variant_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::variant_definition,
        "handle_type_variant_definitions got a non variant_definition"
    );
    todo!()
}

pub fn handle_type_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::type_definition,
        "handle_type_definitions got a non type_definition"
    );
    println!("{pair:?}\n");
    let inner = pair.into_inner().next().unwrap();
    return match inner.as_rule() {
        Rule::struct_definition => {
            todo!()
        }
        Rule::alias_definition => handle_type_alias_definitions(inner),
        Rule::variant_definition => {
            todo!()
        }
        Rule::union_definition => {
            todo!()
        }
        _ => unreachable!(),
    };
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
