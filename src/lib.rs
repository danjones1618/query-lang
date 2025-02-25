use pest::iterators::Pair;
use pest::Parser;
use pyo3::types::IntoPyDict;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn parse_to_string(py: Python, to_parse: &str) -> PyResult<String> {
    let res = match parse_query_string(&to_parse) {
        Ok(e) => format!("{e:?}"),
        Err(e) => format!("{e}"),
    };
    let query_model = py
        .import("django.db.models.sql.query")
        .expect("TODO: handle django not installed");
    let kwargs = vec![("hi", "yes"), ("name__ieq", "hello")].into_py_dict(py)?;
    let q_object = query_model.getattr("Q")?.call((), Some(&kwargs))?;
    let hmm = q_object.to_string();
    Ok(format!("Ooo yaaasss::: {res} \nNooo: {hmm}"))
}

#[pymodule]
#[pyo3(name = "_core")]
fn query_lang(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_to_string, m)?)?;
    Ok(())
}

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
pub enum Expression<'a> {
    QueryItem(QueryItem<'a>),
    CombinedExpression(Combinator<'a>),
}

pub fn parse_query_string(query: &str) -> Result<Expression, String> {
    let mut parsed = match parser::QLParser::parse(parser::Rule::start, query) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{e}"));
        }
    };
    Ok(parser::parse_expression(parsed.next().unwrap()))
}

mod parser {
    use super::*;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "grammer.pest"]
    pub struct QLParser;

    pub fn parse_expression(rule: Pair<Rule>) -> Expression {
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
}
