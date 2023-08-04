use super::*;
use crate::program::expression_if::convert_if;

pub fn convert_expression(exp: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    let exp = exp.as_expression().unwrap();
    match exp {
        Expression::Literal(literal_node) => {
            let (literal_type, literal_value) = literal_node.as_literal().unwrap();
            let value = match literal_type {
                Literal::String => Value::String(snailquote::unescape(literal_value)?),
                Literal::Number => Value::Number(Number::from(literal_value.parse::<i64>()?)),
                Literal::Boolean => Value::Bool(match literal_value {
                    "true" => true,
                    "false" => false,
                    _ => bail!("invalid bool literal, {literal_value}"),
                }),
            };
            operations.push(Op::Load(ReferenceOrValue::Value(value)));
        }
        Expression::Reference(reference) => operations.push(Op::Load(ReferenceOrValue::Reference(
            reference
                .as_reference()
                .unwrap()
                .iter()
                .map(|i| i.as_ident().unwrap())
                .collect::<Vec<_>>()
                .join(".")
                .into(),
        ))),
        Expression::Call(call) => {
            // each call creates a new scope
            operations.push(Op::EnterScope);
            let (path, params) = call.as_call().unwrap();

            let mut param_references: Vec<String> = vec![];

            for (idx, param) in params.iter().enumerate() {
                convert_expression(param, operations)?;

                let param_name = format!("_{idx}");
                operations.push(Op::Declare {
                    name: param_name.clone(),
                    ty: Type::Json,
                });

                param_references.push(param_name.clone());
                operations.push(Op::Store { name: param_name });
            }

            operations.push(Op::Call {
                path: path.as_path().unwrap().to_string(),
                params: param_references
                    .into_iter()
                    .map(|p| ReferenceOrValue::Reference(p))
                    .collect(),
            });

            operations.push(Op::ExitScope);
        }
        Expression::If(if_exp) => {
            convert_if(if_exp, operations)?;
        }
        Expression::Block(block) => {
            convert_block(block, operations)?;
        }
    }
    Ok(())
}
