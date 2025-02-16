use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammer.pest"]
struct QLParser;

#[derive(Debug)]
enum MatchType {
    Eq,
    NotEq,
    NotRegex,
    Regex,
}

impl TryFrom<&str> for MatchType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "!=" => Ok(MatchType::NotEq),
            "=~" => Ok(MatchType::Regex),
            "!~" => Ok(MatchType::NotRegex),
            "=" => Ok(MatchType::Eq),
            _ => Err("Invalid match type string".to_owned()),
        }
    }
}

#[derive(Debug)]
struct QueryItem<'a> {
    attribute: &'a str,
    match_type: MatchType,
    match_value: &'a str,
}

#[derive(Debug)]
enum Combinator<'a> {
    And {
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    Or {
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
}

#[derive(Debug)]
enum Expression<'a> {
    QueryItem(QueryItem<'a>),
    CombinedExpression(Combinator<'a>),
}

fn parse_expression(rule: Pair<Rule>) -> Expression {
    match rule.as_rule() {
        Rule::and_expr => {
            let mut rules = rule.into_inner();
            let lhs = parse_expression(rules.next().unwrap());
            let rhs = parse_expression(rules.next().unwrap());
            Expression::CombinedExpression(Combinator::And {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        Rule::or_expr => {
            let mut rules = rule.into_inner();
            let lhs = parse_expression(rules.next().unwrap());
            let rhs = parse_expression(rules.next().unwrap());
            Expression::CombinedExpression(Combinator::Or {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        Rule::comparison => {
            let mut rules = rule.into_inner();
            let attribute = rules.next().unwrap().as_span().as_str();
            let match_type: MatchType = rules.next().unwrap().as_str().try_into().unwrap();
            let match_value = rules.next().unwrap().as_span().as_str();
            Expression::QueryItem(QueryItem {
                attribute,
                match_type,
                match_value,
            })
        }
        Rule::EOI
        | Rule::WHITESPACE
        | Rule::char
        | Rule::field
        | Rule::excaped_str
        | Rule::op
        | Rule::exchar => {
            unreachable!()
        }
        Rule::start
        | Rule::and_expr_or_comparision
        | Rule::or_expr_or_comparision
        | Rule::expr
        | Rule::bracket_expr => rule
            .into_inner()
            .map(|v| parse_expression(v))
            .next()
            .unwrap(),
    }
}

fn main() {
    let mut parsed = match QLParser::parse(
        Rule::start,
        r#"(name     = "hi ehllo * there _ yaes" or blah =~ "7*") and location = "eu-1" and (yes != "okay" or yes = "hmm nice")"#,
    ) {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    let result: Expression = parse_expression(parsed.next().unwrap());
    println!("Hello, world: {result:?}");
}
