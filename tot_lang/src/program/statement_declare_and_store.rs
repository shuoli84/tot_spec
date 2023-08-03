use super::*;

pub fn convert_declare_and_bind(
    ident: &AstNode,
    path: &AstNode,
    expr: &AstNode,
    operations: &mut Vec<Op>,
) -> anyhow::Result<()> {
    let ident = ident.as_ident().unwrap().to_string();

    let type_path = path.as_path().unwrap();
    let ty = Type::try_parse(type_path)?;

    operations.push(Op::Declare {
        name: ident.clone(),
        ty: ty,
    });

    convert_expression(expr, operations)?;
    operations.push(Op::Store { name: ident });

    Ok(())
}
