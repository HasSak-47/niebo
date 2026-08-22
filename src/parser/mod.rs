mod expressions;
mod type_handling;

use expressions::{handle_block_expression, handle_expression};
use type_handling::*;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::{
    ast::{
        Definition, DefinitionKind, Implementation, Import, TraitImplementation, Visibility,
        expressions::Statement,
        function::{FunctionBuilder, FunctionDeclaration},
        module::{Module, ModuleKind},
    },
    general::{
        naming::{QualifiedName, QualifiedNameSegment},
        types::Type,
    },
};

#[derive(Parser)]
#[grammar = "./pest/tokens.pest"]
pub struct TokenStream;

pub fn handle_param<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<(String, Type)> {
    assert_eq!(pair.as_rule(), Rule::param);
    let mut inner = pair.into_inner();

    let id_p = inner.next().unwrap();
    assert_eq!(id_p.as_rule(), Rule::identifier);

    let pt_p = inner.next().unwrap();
    assert_eq!(pt_p.as_rule(), Rule::type_expr);

    return Ok((id_p.as_str().to_string(), handle_type_expression(pt_p)?));
}

pub fn handle_params<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<(String, Type)>> {
    assert_eq!(pair.as_rule(), Rule::params);
    let inner = pair.into_inner();
    let mut v = Vec::new();

    for param in inner {
        let (ident, ty) = handle_param(param)?;
        v.push((ident, ty));
    }

    return Ok(v);
}

pub fn handle_fn_params<'a>(
    pair: Pair<'a, Rule>,
    mut builder: FunctionBuilder,
) -> anyhow::Result<FunctionBuilder> {
    assert_eq!(pair.as_rule(), Rule::params);
    for (ident, ty) in handle_params(pair)? {
        builder = builder.add_param(ident, ty);
    }

    return Ok(builder);
}

pub fn handle_template_def<'a>(_pair: Pair<'a, Rule>) -> anyhow::Result<()> {
    todo!()
}

pub fn handle_fn_declaration<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<FunctionBuilder> {
    let mut inner = pair.into_inner().into_iter();
    let ident = inner.next().unwrap();
    let mut builder = FunctionBuilder::new(ident.as_str());
    let next = inner.next().unwrap();
    match next.as_rule() {
        Rule::template_def => {
            handle_template_def(next)?;
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
        Rule::return_statement => {
            let mut expr = next.into_inner();
            let value = expr.next().and_then(|k| handle_expression(k).ok());
            Statement::Return(value)
        }
        Rule::break_statement => {
            let mut break_ = next.into_inner();
            if let Some(name) = break_.next() {
                Statement::Break(Some(name.as_str().to_string()))
            } else {
                Statement::Break(None)
            }
        }
        Rule::import | Rule::const_definition | Rule::continue_statement => todo!(),
        un => unreachable!("{un:?}"),
    });
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
    let function = handle_fn_declaration(declaration)?.set_body(handle_block_expression(block)?);

    return Ok(function.build_def());
}

pub fn handle_trait_definitions<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(pair.as_rule(), Rule::trait_definition);
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str().to_string();
    let mut functions = std::collections::HashMap::new();

    for item in inner {
        match item.as_rule() {
            Rule::template_def => {}
            Rule::trait_body => {
                let body = item.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::fn_declaration => {
                        let builder = handle_fn_declaration(body)?;
                        let name = builder.ident;
                        let function = crate::ast::function::FunctionDefinition {
                            def: FunctionDeclaration {
                                constant: builder.constant,
                                return_ty: builder.ret_ty,
                                parameters: builder
                                    .params
                                    .into_iter()
                                    .map(|(name, ty)| (name.unwrap(), ty))
                                    .collect(),
                            },
                            body: crate::ast::expressions::block::Block::new(),
                        };
                        functions.insert(name, function);
                    }
                    Rule::fn_definition => {
                        let def = handle_fn_definitions(body)?;
                        if let DefinitionKind::FunctionDefinition(function) = def.kind {
                            functions.insert(def.name, function);
                        }
                    }
                    Rule::type_definition | Rule::type_declaration => {}
                    un => unreachable!("{un:?}"),
                }
            }
            un => unreachable!("{un:?}"),
        }
    }

    Ok(Definition {
        kind: DefinitionKind::Trait(crate::ast::traits::Trait { functions }),
        visibility: Visibility::Public,
        name: ident,
    })
}

pub fn handle_path_ident<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<QualifiedNameSegment> {
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str().to_string();
    if let Some(_) = inner.next() {
        todo!("handle template specialization");
    }

    return Ok(QualifiedNameSegment {
        ident,
        template_spec: vec![],
    });
}

pub fn handle_qualified_name<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<QualifiedName> {
    let inner = pair.into_inner();
    let next = inner.peek().unwrap();

    if let Rule::rel_path = next.as_rule() {
        todo!("handle relative paths...");
    }

    let mut path = QualifiedName::new();
    for ident in inner {
        path.add_segment(handle_path_ident(ident)?);
    }

    return Ok(path);
}

pub fn handle_import_path<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<QualifiedName>> {
    assert_eq!(pair.as_rule(), Rule::import_path);

    let inner = pair.into_inner().next().unwrap();
    return match inner.as_rule() {
        Rule::path => Ok(vec![handle_qualified_name(inner)?]),
        Rule::grouped_import_path => handle_grouped_import_path(inner),
        _ => unreachable!(),
    };
}

pub fn handle_grouped_import_path<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<QualifiedName>> {
    assert_eq!(pair.as_rule(), Rule::grouped_import_path);

    let mut inner = pair.into_inner();
    let prefix = handle_qualified_name(inner.next().unwrap())?;
    let group = inner.next().unwrap();
    assert_eq!(group.as_rule(), Rule::import_group);

    let mut imports = Vec::new();
    for segment in group.into_inner() {
        let mut path = prefix.clone();
        path.add_segment(handle_path_ident(segment)?);
        imports.push(path);
    }

    return Ok(imports);
}

pub fn handle_implementation<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Implementation> {
    assert_eq!(pair.as_rule(), Rule::implementation);
    let mut inner = pair.into_inner();
    let target = handle_qualified_name(inner.next().unwrap())?;
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
    let target = handle_qualified_name(inner.next().unwrap())?;
    let trait_path = handle_qualified_name(inner.next().unwrap())?;
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
        todo!("handle infile module");
    }

    let mut md = Module::new();
    md.kind = ModuleKind::ExFile;

    return Ok(Definition::module(ident.as_str(), md));
}

pub fn handle_definition<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Definition> {
    assert_eq!(
        pair.as_rule(),
        Rule::definition,
        "a non Rule::definition reached handle_definitions"
    );
    let mut inner = pair.into_inner();

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

pub fn handle_c_imports<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<Import>> {
    let inner = pair.into_inner().next().unwrap();
    assert_eq!(inner.as_rule(), Rule::import_path);

    return Ok(handle_import_path(inner)?
        .into_iter()
        .map(Import::c_import)
        .collect());
}

pub fn handle_niebo_imports<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<Import>> {
    let inner = pair.into_inner().next().unwrap();
    assert_eq!(inner.as_rule(), Rule::import_path);

    return Ok(handle_import_path(inner)?
        .into_iter()
        .map(Import::niebo_import)
        .collect());
}

pub fn handle_imports<'a>(pair: Pair<'a, Rule>) -> anyhow::Result<Vec<Import>> {
    let inner = pair.into_inner().next().unwrap();
    return match inner.as_rule() {
        Rule::c_import => handle_c_imports(inner),
        Rule::niebo_import => handle_niebo_imports(inner),
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
            Rule::definition => {
                md.definitions.push(handle_definition(t)?);
            }
            Rule::import => {
                md.imports.extend(handle_imports(t)?);
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
    use anyhow::bail;

    use super::*;
    use crate::ast::expressions::intrinsic::IntrinsicKind;
    use crate::ast::expressions::literal::*;
    use crate::ast::expressions::operations::*;
    use crate::ast::expressions::*;
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
            Expression::method_call(Expression::identifier("c"), "d", vec![]),
        );

        assert!(parsed == expected, "{parsed} != {expected}");

        return Ok(());
    }

    #[test]
    fn test_member_access() -> anyhow::Result<()> {
        //  TODO: check that the postfix is correct
        // expression postfix:call postfix:access postfix:call
        let access = TokenStream::parse(Rule::expression, "(a().b)(10)")?;
        let exp = handle_expression(access.into_iter().next().unwrap())?;

        if let ExpressionKind::Call(call) = &*exp.kind {
            if let ExpressionKind::MemberAccess(member) = &*call.called.kind {
                assert!(member.member == QualifiedNameSegment::from("b"));
                if let ExpressionKind::Call(called2) = &*member.object.kind {
                    assert!(matches!(
                        &*called2.called.kind,
                        ExpressionKind::Identifier(_)
                    ))
                } else {
                    bail!("b was not a member access from a()")
                }
            } else {
                bail!("called was not (a().b)")
            }
        } else {
            bail!("there was no (a().b)(10) call!")
        }

        return Ok(());
    }

    #[test]
    fn test_method_call() -> anyhow::Result<()> {
        //  TODO: check that the postfix is correct
        // expression postfix:call postfix:access postfix:call
        let access = TokenStream::parse(Rule::expression, "a().b(10)")?;
        let exp = handle_expression(access.into_iter().next().unwrap())?;

        if let ExpressionKind::MethodCall(method) = &*exp.kind {
            assert!(method.method == QualifiedNameSegment::from("b"));
            if let ExpressionKind::Call(called2) = &*method.object.kind {
                assert!(matches!(
                    &*called2.called.kind,
                    ExpressionKind::Identifier(_)
                ))
            } else {
                bail!("b(10) was not a method call from a()")
            }
        } else {
            bail!("there was no a().b(10) method call!")
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
            Rule::definition,
            "inter TestTrait<T: A<T>>{\n\ttype DeclaredType = T;\n\ttype DefinedType = T;\n\tfn func() -> Type;\n}",
        )?;

        TokenStream::parse(
            Rule::trait_definition,
            "inter testTrait<T: Add<T>>{
    type TestType = i32;
    
    fn test_function<T: Add<T>>(t: T) -> T ;
}",
        )?;

        return Ok(());
    }

    #[test]
    fn test_extend_impl() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::trait_implementation,
            "extend Vec2 as Add<Vec2> {
    fn add(other: Vec2) -> Vec2 {
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
    fn test_intrinsic_expression() -> anyhow::Result<()> {
        let expression = TokenStream::parse(Rule::expression, "@add_i32(self, other)")?
            .next()
            .unwrap();
        let parsed = handle_expression(expression)?;

        match parsed.kind.as_ref() {
            ExpressionKind::Intrinsic(intrinsic) => {
                assert_eq!(intrinsic.kind, IntrinsicKind::AddI { prec: 32 });
                assert_eq!(intrinsic.parameters.len(), 2);
            }
            other => panic!("expected intrinsic expression, got {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_conversion_intrinsic_expression() -> anyhow::Result<()> {
        let expression = TokenStream::parse(Rule::expression, "@i32_to_u32(self)")?
            .next()
            .unwrap();
        let parsed = handle_expression(expression)?;

        match parsed.kind.as_ref() {
            ExpressionKind::Intrinsic(intrinsic) => {
                assert_eq!(
                    intrinsic.kind,
                    IntrinsicKind::IToU {
                        src_prec: 32,
                        out_prec: 32,
                    }
                );
                assert_eq!(intrinsic.parameters.len(), 1);
            }
            other => panic!("expected intrinsic expression, got {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_neg_intrinsic_expression() -> anyhow::Result<()> {
        let expression = TokenStream::parse(Rule::expression, "@neg_i32(self)")?
            .next()
            .unwrap();
        let parsed = handle_expression(expression)?;

        match parsed.kind.as_ref() {
            ExpressionKind::Intrinsic(intrinsic) => {
                assert_eq!(intrinsic.kind, IntrinsicKind::NegI { prec: 32 });
                assert_eq!(intrinsic.parameters.len(), 1);
            }
            other => panic!("expected intrinsic expression, got {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_copy_intrinsic_expression() -> anyhow::Result<()> {
        let expression = TokenStream::parse(Rule::expression, "@copy(i32)")?
            .next()
            .unwrap();
        let parsed = handle_expression(expression)?;

        match parsed.kind.as_ref() {
            ExpressionKind::Intrinsic(intrinsic) => {
                assert_eq!(intrinsic.kind, IntrinsicKind::Copy);
                assert_eq!(intrinsic.parameters.len(), 1);
            }
            other => panic!("expected intrinsic expression, got {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn test_grouped_header_import() -> anyhow::Result<()> {
        let module = parse_module("header stdio::{printf, scanf, };\n")?;

        assert_eq!(module.imports.len(), 2);
        assert!(module.imports.iter().all(|import| import.c_import));
        assert_eq!(format!("{:?}", module.imports[0].path), "stdio::printf");
        assert_eq!(format!("{:?}", module.imports[1].path), "stdio::scanf");

        Ok(())
    }

    #[test]
    fn test_grouped_niebo_import() -> anyhow::Result<()> {
        let module = parse_module("import core::{i32, traits, };\n")?;

        assert_eq!(module.imports.len(), 2);
        assert!(module.imports.iter().all(|import| !import.c_import));
        assert_eq!(format!("{:?}", module.imports[0].path), "core::i32");
        assert_eq!(format!("{:?}", module.imports[1].path), "core::traits");

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
    fn test_break() -> anyhow::Result<()> {
        let mut stream = TokenStream::parse(
            Rule::block_expression,
            "{
    scanf(\"%d\", &i);

    if i == 0{
        printf(\"exiting...\");
        break;
    }
}",
        )?
        .next()
        .unwrap()
        .into_inner();

        stream.next().unwrap();
        let statement_if_ = stream.next().unwrap();
        let expression_if_ = statement_if_.into_inner().next().unwrap();
        let mut if_ = expression_if_.into_inner().next().unwrap().into_inner();
        let if_con = if_.next().unwrap();
        assert_eq!(if_con.as_rule(), Rule::expression);
        let if_then = if_.next().unwrap();
        assert_eq!(if_then.as_rule(), Rule::block_expression);
        let exp = handle_block_expression(if_then)?;
        if let Statement::Break(None) = exp.statements[1] {
            return Ok(());
        }
        anyhow::bail!("break statement not found");
    }

    #[test]
    fn test_min_clike() -> anyhow::Result<()> {
        TokenStream::parse(
            Rule::module,
            "header stdio::printf;

type TestType = i32;

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
