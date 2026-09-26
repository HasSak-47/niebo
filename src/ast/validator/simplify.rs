use crate::ast::{
    DefinitionKind,
    expressions::{
        Expression,
        ExpressionKind::{self},
        Statement,
        block::Block,
        call::Call,
        conditional::ConditionalBuilder,
        member_access::MethodCall,
    },
    project::Project,
};

use super::Validator;

impl Validator {
    fn simplify_expression(&self, expr: &mut Expression) -> Option<Expression> {
        match expr.kind.as_mut() {
            ExpressionKind::BinaryOperation(bin) => Some(Expression::call(
                Expression::identifier(bin.operator.into_operation_call()),
                bin.operands.to_vec(),
            )),
            ExpressionKind::UnaryOperation(un) => Some(Expression::call(
                Expression::identifier(un.operator.into_operation_call()),
                vec![un.operand.clone()],
            )),
            ExpressionKind::While(while_loop) => {
                let mut else_body = Block::new();
                else_body.statements.push(Statement::Break(None));

                let cond = Expression::if_(
                    ConditionalBuilder::new(while_loop.condition.clone(), while_loop.then.clone())
                        .set_else(Expression::block(else_body))
                        .build(),
                );
                let mut loop_body = Block::new();
                loop_body.add_statement(Statement::Expression(cond));

                return Some(Expression::loop_(Expression::block(loop_body), None));
            }
            ExpressionKind::Block(block) => {
                return Some(Expression::block(self.simplify_block(block.clone())));
            }
            ExpressionKind::Index(val, index) => {
                self.simplify_expression(val)?;
                self.simplify_expression(index)?;
                return Some(Expression::index_access(val.clone(), index.clone()));
            }
            ExpressionKind::If(cond) => {
                let mut builder = ConditionalBuilder::new(
                    self.simplify_expression(&mut cond.condition)?.clone(),
                    self.simplify_expression(&mut cond.then)?.clone(),
                );
                if let Some(else_) = &mut cond.else_ {
                    builder = builder.set_else(self.simplify_expression(else_)?.clone());
                }

                return Some(Expression::if_(builder.build()));
            }
            ExpressionKind::Call(Call { called, parameters }) => {
                let mut params = Vec::new();
                for param in parameters {
                    params.push(self.simplify_expression(param)?.clone());
                }
                return Some(Expression::call(
                    self.simplify_expression(called)?.clone(),
                    params,
                ));
            }

            ExpressionKind::MethodCall(MethodCall {
                object,
                method,
                params,
            }) => {
                let mut params_ = Vec::new();

                params_.push(self.simplify_expression(object)?.clone());
                for param in params {
                    params_.push(self.simplify_expression(param)?.clone());
                }

                todo!();
                //let ty = object.ret_ty.unwrap();
                // return Some(Expression::call(
                //     self.simplify_expression(called)?.clone(),
                //     params_,
                // ));
            }
            _ => todo!(),
        }
    }

    fn simplify_block(&self, mut block: Block) -> Block {
        for mut stmt in &mut block.statements {
            match &mut stmt {
                Statement::Expression(exp) => {
                    if let Some(new_exp) = self.simplify_expression(exp) {
                        *stmt = Statement::Expression(new_exp);
                    }
                }
                _ => {}
            }
        }

        return block;
    }

    /*
    makes a project ast to have less fluff like operations, whiles, etc etc
    */
    pub fn simplify_project(&self, project: &mut Project) {
        for def in &mut project.root_module.definitions {
            match &mut def.kind {
                DefinitionKind::FunctionDefinition(func) => {
                    func.body = self.simplify_block(func.body.clone());
                }
                _ => {}
            }
        }
    }
}
