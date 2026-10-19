use pest::iterators::Pair;

use crate::{
    ast::{Definition, Visibility},
    general::types::typeid::{PrimitiveTypeID, StructTypeID, TypeID},
    parser::*,
};

fn handle_primitive_type<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<PrimitiveTypeID> {
    assert_eq!(
        pair.as_rule(),
        Rule::primitive_type,
        "handle_primitive_type got a non primitive_type"
    );
    let raw = pair.as_str();
    let mut inner = pair.into_inner();
    let Some(next) = inner.next() else {
        return Ok(match raw {
            "bool" => PrimitiveTypeID::Bool,
            "int" => PrimitiveTypeID::Int(32),
            un => unreachable!("{un:?}"),
        });
    };
    match next.as_rule() {
        Rule::int_type => {
            let prec = next.into_inner().next().unwrap().as_str().parse()?;
            return Ok(PrimitiveTypeID::Int(prec));
        }
        Rule::uint_type => {
            let prec = next.into_inner().next().unwrap().as_str().parse()?;
            return Ok(PrimitiveTypeID::Uint(prec));
        }
        Rule::float_type => {
            let prec = next.into_inner().next().unwrap().as_str().parse()?;
            return Ok(PrimitiveTypeID::Float(prec));
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
        ident.clone(),
        TypeID::alias(ident.into(), handle_type_expression(type_exp)?),
        Visibility::Public,
    ));
}

pub fn handle_type_struct_declaration<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<String> {
    assert_eq!(
        pair.as_rule(),
        Rule::struct_declaration,
        "handle_type_struct_definitions got a non struct_definition"
    );
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap();
    assert_eq!(ident.as_rule(), Rule::identifier);

    return Ok(ident.as_str().to_string());
}

pub fn handle_type_struct_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::struct_definition,
        "handle_type_struct_definitions got a non struct_definition"
    );

    let mut inner = pair.into_inner();

    let ident = inner.next().unwrap().as_str().to_string();
    let mut members = Vec::new();

    let fields = inner.next().unwrap();
    let params = handle_params(fields)?;
    for param in params {
        members.push(param);
    }

    return Ok(Definition::type_def(
        ident,
        TypeID::struct_t(members),
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

pub fn handle_type_expression<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<TypeID> {
    assert_eq!(
        pair.as_rule(),
        Rule::type_expr,
        "handle_type_expression got a non type_expr"
    );
    let mut inner = pair.into_inner();
    let ty = inner.next().unwrap();
    return match ty.as_rule() {
        Rule::primitive_type => Ok(TypeID::primitive(handle_primitive_type(ty)?)),
        Rule::mutable_reference_type => {
            let mut inner = ty.into_inner();
            let next = inner.next().unwrap();
            Ok(TypeID::mutable_reference(handle_type_expression(next)?))
        }
        Rule::mutable_pointer_type => {
            let mut inner = ty.into_inner();
            let next = inner.next().unwrap();
            Ok(TypeID::mutable_pointer(handle_type_expression(next)?))
        }
        Rule::path => Ok(TypeID::named(handle_qualified_name(ty)?)),

        un => unreachable!("{un:?}"),
    };
}
