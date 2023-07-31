use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
pub struct GrammarParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_grammar() {
        let successful_parse = GrammarParser::parse(Rule::file, "fn hello() {}").unwrap();
        for file in successful_parse {
            for func in file.into_inner() {
                if !matches!(func.as_rule(), Rule::func) {
                    continue;
                }

                let mut inner_rules = func.into_inner();
                let func_signature = inner_rules.next();
                let func_body = inner_rules.next();

                dbg!(func_signature);
                dbg!(func_body);
            }
        }
    }
}
