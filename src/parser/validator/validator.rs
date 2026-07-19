use super::{Symbol, Validator};
use anyhow::bail;

use crate::{
    ast::expressions::{Statement, loops::WhileLoop, operations::BinaryOperation},
    general::types::{PrimitiveType, Type},
};

pub(super) trait ExpressionValidator {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()>;
    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type>;
}

impl ExpressionValidator for crate::ast::expressions::loops::LoopExpression {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        self.body.validate(procesor)
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        self.body.resolve_ret_ty(procesor)
    }
}

impl ExpressionValidator for crate::ast::expressions::operations::UnaryOperation {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        use crate::ast::expressions::operations::UnaryOperator;
        self.operand.validate(procesor)?;
        let operand_ty = self.operand.resolve_ret_ty(procesor)?;
        match self.operator {
            UnaryOperator::EarlyRet => {
                return Ok(());
            }
            UnaryOperator::Increase => {
                return Ok(());
            }
            UnaryOperator::Decrease => {
                return Ok(());
            }
            UnaryOperator::Ref => {
                return Ok(());
            }
            UnaryOperator::Deref => match operand_ty {
                Type::Pointer(_) => return Ok(()),
                ty => bail!("{ty:?} cannot be deref"),
            },
            UnaryOperator::Negation => {
                todo!()
            }
        }
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        use crate::ast::expressions::operations::UnaryOperator;
        let oper_ty = self.operand.resolve_ret_ty(procesor)?;
        match self.operator {
            UnaryOperator::Ref => {
                return Ok(Type::pointer(oper_ty));
            }
            UnaryOperator::Increase => {
                return Ok(Type::int());
            }
            UnaryOperator::Decrease => {
                return Ok(Type::int());
            }
            UnaryOperator::EarlyRet => {
                todo!()
            }
            UnaryOperator::Deref => match self.operand.resolve_ret_ty(procesor)? {
                Type::Pointer(ty) => return Ok(*ty),
                ty => unreachable!("{ty:?} cannot be deref"),
            },
            UnaryOperator::Negation => {
                todo!()
            }
        }
    }
}

impl ExpressionValidator for crate::ast::expressions::literal::Literal {
    fn resolve_ret_ty(&mut self, _: &mut Validator) -> anyhow::Result<Type> {
        use crate::ast::expressions::literal::LiteralInfo;
        return Ok(match self.info {
            LiteralInfo::String => Type::string(),
            LiteralInfo::Integer { .. } => Type::int(),
            LiteralInfo::Float { .. } => Type::float(),
        });
    }

    fn validate(&mut self, _: &mut Validator) -> anyhow::Result<()> {
        return Ok(());
    }
}

impl ExpressionValidator for crate::ast::expressions::call::Call {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        self.called.validate(procesor)?;
        for param in &mut self.parameters {
            param.validate(procesor)?;
        }

        return Ok(());
    }
    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        let function_type = self.called.resolve_ret_ty(procesor)?;
        for param in &mut self.parameters {
            param.resolve_ret_ty(procesor)?;
        }
        if let Type::Function(f) = function_type {
            return Ok(f.ret_ty.as_ref().clone());
        }
        return Err(anyhow::anyhow!(
            "tried to call non function type: {function_type:?}"
        ));
    }
}

impl ExpressionValidator for crate::general::naming::QualifiedName {
    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        let ty = procesor
            .find_symbol(self.clone())
            .ok_or(anyhow::anyhow!("failed to find symbol: {}", self))?;
        match ty {
            Symbol::Type { ty, .. } => return Ok(ty),
            Symbol::Variable(ty) => return Ok(ty),
            Symbol::Function { ret_ty, params } => {
                return Ok(Type::function(params, ret_ty, false));
            }
            td => todo!("{td:?}"),
        }
    }

    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        procesor
            .find_symbol(self.clone().into())
            .ok_or(anyhow::anyhow!("failed to find symbol: {}", self))?;

        return Ok(());
    }
}

impl ExpressionValidator for crate::ast::expressions::conditional::Conditional {
    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        return self.then.resolve_ret_ty(procesor);
    }

    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        self.condition.validate(procesor)?;
        self.then.validate(procesor)?;
        match &mut self.else_ {
            Some(s) => {
                s.validate(procesor)?;
                if s.resolve_ret_ty(procesor)? != self.then.resolve_ret_ty(procesor)? {
                    bail!("branches are of different type");
                }
            }
            _ => {}
        }
        if let Type::Primitive(PrimitiveType::Bool) = self.condition.resolve_ret_ty(procesor)? {
            return Ok(());
        }
        bail!("if condition type is not a boolean");
    }
}

impl ExpressionValidator for crate::ast::expressions::Expression {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        use crate::ast::expressions::ExpressionKind;

        match self.kind.as_mut() {
            ExpressionKind::BinaryOperation(b_exp) => b_exp.validate(procesor),
            ExpressionKind::While(w_exp) => w_exp.validate(procesor),
            ExpressionKind::Identifier(ident) => ident.validate(procesor),
            ExpressionKind::Literal(lit) => lit.validate(procesor),
            ExpressionKind::Block(blk) => blk.validate(procesor),
            ExpressionKind::Call(call) => call.validate(procesor),
            ExpressionKind::Assignment(a, b) => {
                a.validate(procesor)?;
                b.validate(procesor)?;
                let aty = a.resolve_ret_ty(procesor)?;
                let bty = b.resolve_ret_ty(procesor)?;

                if aty != bty {
                    bail!("Assignment ({a} = {b}) = {aty:?} is different from {bty:?}");
                }

                Ok(())
            }
            ExpressionKind::UnaryOperation(unary) => unary.validate(procesor),
            ExpressionKind::Loop(loop_) => loop_.validate(procesor),
            ExpressionKind::If(if_) => if_.validate(procesor),
            ExpressionKind::Index(exp, index) => {
                exp.validate(procesor)?;
                index.validate(procesor)?;

                if let Some(ty) = &exp.ret_ty {
                    match ty {
                        Type::Array(_) => {}
                        Type::Pointer(_) => {}
                        Type::MutablePointer(_) => {}
                        _ => bail!("cannot index non array types"),
                    }
                }

                if let Some(ty) = &index.ret_ty {
                    match ty {
                        Type::Primitive(PrimitiveType::Uint(_))
                        | Type::Primitive(PrimitiveType::Int(_)) => {}
                        _ => bail!("cannot index with non int types: {index:?}"),
                    }
                }

                Ok(())
            }
            td => todo!("{td:?}"),
        }
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        use crate::ast::expressions::ExpressionKind;
        if self.ret_ty.is_none() {
            let ty = match self.kind.as_mut() {
                ExpressionKind::BinaryOperation(b_exp) => b_exp.resolve_ret_ty(procesor),
                ExpressionKind::While(w_exp) => w_exp.resolve_ret_ty(procesor),
                ExpressionKind::Identifier(ident) => ident.resolve_ret_ty(procesor),
                ExpressionKind::Literal(lit) => lit.resolve_ret_ty(procesor),
                ExpressionKind::Block(blk) => blk.resolve_ret_ty(procesor),
                ExpressionKind::Call(call) => call.resolve_ret_ty(procesor),

                ExpressionKind::Assignment(a, _) => a.resolve_ret_ty(procesor),
                ExpressionKind::UnaryOperation(unary) => unary.resolve_ret_ty(procesor),
                ExpressionKind::Loop(loop_) => loop_.resolve_ret_ty(procesor),
                ExpressionKind::If(if_) => if_.resolve_ret_ty(procesor),
                ExpressionKind::Index(exp, index) => {
                    index.resolve_ret_ty(procesor)?;
                    match exp.resolve_ret_ty(procesor)? {
                        Type::Array(t) => Ok(t.as_ref().clone()),
                        Type::Pointer(t) => Ok(t.as_ref().clone()),
                        Type::MutablePointer(t) => Ok(t.as_ref().clone()),
                        u => unreachable!("{u:?}"),
                    }
                }
                _ => todo!(),
            }?;
            self.ret_ty = Some(ty);
        }

        return Ok(self.ret_ty.clone().unwrap());
    }
}

impl ExpressionValidator for crate::ast::expressions::block::Block {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        use crate::ast::{Definition, DefinitionKind, expressions::Statement};

        procesor.push_scope();
        for stmt in &mut self.statements {
            match stmt {
                Statement::Definition(Definition { kind, name, .. }) => match kind {
                    DefinitionKind::Function(func) => {
                        procesor.register_local_symbol(
                            name.clone().into(),
                            Symbol::Function {
                                ret_ty: func.return_ty.clone().unwrap(),
                                params: func.parameters.iter().map(|a| a.1.clone()).collect(),
                            },
                        );
                    }
                    DefinitionKind::Variable(var) => {
                        var.value.validate(procesor)?;
                        println!("{name} = {var:#?}");
                        procesor.register_local_symbol(
                            name.clone().into(),
                            Symbol::Variable(var.ty.clone().expect(&format!("stmt: {stmt:#?}"))),
                        );
                    }
                    _ => todo!(),
                },
                Statement::Expression(ex) => {
                    ex.validate(procesor)?;
                }
                Statement::Return(ex) => {
                    match ex {
                        Some(s) => s.validate(procesor)?,
                        _ => {}
                    }

                    return Ok(());
                }
                // WARN: BAD! procesor is not aware when is inside a breakable block!
                Statement::Break(_) => {
                    return Ok(());
                }
                td => todo!("{td:?}"),
            }
        }
        procesor.pop_scope();

        return Ok(());
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        for stmt in &mut self.statements {
            match stmt {
                Statement::Expression(exp) => {
                    exp.validate(procesor)?;
                    exp.resolve_ret_ty(procesor)?;
                }
                _ => {}
            }
        }
        return Ok(Type::void());
    }
}

impl ExpressionValidator for WhileLoop {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        self.condition.validate(procesor)?;
        self.then.validate(procesor)?;
        let ty = self.condition.resolve_ret_ty(procesor)?;
        if ty != Type::bool() {
            bail!("condition is not boolean type");
        }

        return Ok(());
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        return self.then.resolve_ret_ty(procesor);
    }
}

impl ExpressionValidator for BinaryOperation {
    fn validate(&mut self, procesor: &mut Validator) -> anyhow::Result<()> {
        self.operands[0].validate(procesor)?;
        self.operands[1].validate(procesor)?;

        let a_ty = self.operands[0].resolve_ret_ty(procesor)?;
        let b_ty = self.operands[1].resolve_ret_ty(procesor)?;
        if a_ty.is_pointer() && b_ty.is_pointer() {
            return Ok(());
        }

        if a_ty != b_ty {
            anyhow::bail!(
                "{}::{a_ty:?} and {}::{b_ty:?} are not the same type",
                self.operands[0],
                self.operands[1]
            );
        }
        return Ok(());
    }

    fn resolve_ret_ty(&mut self, procesor: &mut Validator) -> anyhow::Result<Type> {
        use crate::ast::expressions::operations::BinaryOperator;
        self.validate(procesor)?;
        return Ok(match self.operator {
            BinaryOperator::Greater
            | BinaryOperator::Lesser
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::LesserOrEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual => Type::bool(),
            _ => self.operands[0].ret_ty.clone().unwrap(),
        });
    }
}
