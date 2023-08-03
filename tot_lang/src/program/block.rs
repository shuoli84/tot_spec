use super::*;
use crate::program::expression::convert_expression;
use crate::program::statement::convert_statement;

pub fn convert_block(block: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    let Some((statements, value_expr)) = block.as_block() else {
        bail!("node is block");
    };

    operations.push(Op::EnterScope);

    for statement in statements {
        convert_statement(statement, operations)?;
    }
    if let Some(value_expr) = value_expr {
        convert_expression(value_expr, operations)?;
    }

    operations.push(Op::ExitScope);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_block() {
        let ast = AstNode::parse_statement(
            r#"{
            let i: i32 = 1;
            let j: i64 = 2;
            let k: i32 = {
                100
            };
            {
                // do nothing
            };
            i
        };"#,
        )
        .unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
    }
}
