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
        let parsed = GrammarParser::parse(Rule::file, include_str!("example.tot"));
        assert!(parsed.is_ok());
    }
}
