use super::*;

pub fn convert_if(if_exp: &AstNode, operations: &mut Vec<Op>) -> anyhow::Result<()> {
    let (condition_exp, block, else_block) = if_exp
        .as_if()
        .ok_or_else(|| anyhow!("expression is not if"))?;

    operations.push(Op::EnterScope);

    convert_expression(condition_exp, operations)?;

    // the count will be updated after block generated
    operations.push(Op::JumpIfFalse(0));

    let current_len = operations.len();
    let jump_idx = current_len - 1;
    convert_block(block, operations)?;
    // add an op to jump else block
    operations.push(Op::Jump(0));

    let block_op_count = operations.len() - current_len;

    operations[jump_idx] = Op::JumpIfFalse(block_op_count);

    if let Some(else_block) = else_block {
        let jump_else_op_idx = operations.len() - 1;
        let current_len = operations.len();
        convert_block(else_block, operations)?;
        let else_block_len = operations.len() - current_len;
        operations[jump_else_op_idx] = Op::Jump(else_block_len);
    }

    operations.push(Op::ExitScope);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_if() {
        let ast = AstNode::parse_statement(
            r#"{
            if 1 {
                100
            } else {
                200
            };
        };"#,
        )
        .unwrap();

        let mut operations = vec![];
        assert!(convert_statement(&ast, &mut operations).is_ok());
        dbg!(operations);
    }
}
