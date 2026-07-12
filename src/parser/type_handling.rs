use pest::iterators::Pair;

use crate::{
    ast::{Definition, Visibility},
    general::types::{PrimitiveType, StructType, Type},
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
        Rule::int_type => {
            let prec = next.into_inner().next().unwrap().as_str().parse()?;
            return Ok(PrimitiveType::Int(prec));
        }
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
    let ident = inner.next().unwrap().as_str().to_string();
    let type_exp = inner.next().unwrap();

    return Ok(Definition::type_def(
        ident,
        Type::Alias(Box::new(handle_type_expression(type_exp)?)),
        Visibility::Public,
    ));
}

pub fn handle_type_struct_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::struct_definition,
        "handle_type_struct_definitions got a non struct_definition"
    );

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap();
    assert_eq!(name.as_rule(), Rule::identifier);
    let ident = name.as_str().to_string();
    let mut s = StructType::default();

    let mut fields = inner.next().unwrap().into_inner();
    handle_fn_params(pair, builder)

    return Ok(Definition::type_def(
        ident,
        Type::Struct(s),
        Visibility::Public,
    ));
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
    let inner = pair.into_inner().next().unwrap();
    return match inner.as_rule() {
        Rule::struct_definition => handle_type_struct_definitions(inner),
        Rule::alias_definition => handle_type_alias_definitions(inner),
        Rule::variant_definition => handle_type_variant_definitions(inner),
        Rule::union_definition => handle_type_union_definitions(inner),
        _ => unreachable!(),
    };
}

pub fn handle_type_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Type> {
    assert_eq!(
        pair.as_rule(),
        Rule::type_expr,
        "handle_type_expression got a non type_expr"
    );
    let mut inner = pair.into_inner();
    let ty = inner.next().unwrap();
    return match ty.as_rule() {
        Rule::primitive_type => Ok(Type::Primitive(handle_primitive_type(ty)?)),
        Rule::mutable_reference_type => {
            let mut inner = ty.into_inner();
            let next = inner.next().unwrap();
            Ok(Type::MutableReference(Box::new(handle_type_expression(
                next,
            )?)))
        }
        Rule::mutable_pointer_type => {
            let mut inner = ty.into_inner();
            let next = inner.next().unwrap();
            Ok(Type::MutablePointer(Box::new(handle_type_expression(
                next,
            )?)))
        }
        Rule::path => Ok(Type::named(handle_path(ty)?)),

        un => unreachable!("{un:?}"),
    };
}
