use pest::error::Error as PestError;
use pest::iterators::Pair;
use pest::Parser;
use pyo3::exceptions::PyException;
use pyo3::types::IntoPyDict;

use pyo3::prelude::*;

#[pyclass(extends=PyException, module="query_lang._core", subclass)]
pub struct ParsingError {
    #[pyo3(get, set)]
    message: String,
}

impl ParsingError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

#[pymethods]
impl ParsingError {
    #[new]
    fn py_new(message: String) -> PyResult<Self> {
        Ok(Self { message })
    }
    fn __str__(&self) -> String {
        self.to_string()
    }
}

impl ToString for ParsingError {
    fn to_string(&self) -> String {
        self.message.to_owned()
    }
}

impl From<PestError<parser::Rule>> for ParsingError {
    fn from(error: PestError<parser::Rule>) -> Self {
        Self::new(error.to_string())
    }
}

impl From<ParsingError> for PyErr {
    fn from(error: ParsingError) -> PyErr {
        Python::with_gil(|py| match Bound::new(py, error) {
            Ok(parse_error) => PyErr::from_value(parse_error.into_any()),
            Err(e) => e,
        })
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn parse_to_string(py: Python, to_parse: &str) -> PyResult<String> {
    let res = parse_query_string(&to_parse)?;
    let query_model = py
        .import("django.db.models.sql.query")
        .expect("TODO: handle django not installed");
    let kwargs = vec![("hi", "yes"), ("name__ieq", "hello")].into_py_dict(py)?;
    let q_object = query_model.getattr("Q")?.call((), Some(&kwargs))?;
    let hmm = q_object.to_string();
    Ok(format!("Ooo yaaasss::: {res:?} \nNooo: {hmm}"))
}

#[pymodule]
#[pyo3(name = "_core")]
fn query_lang_python_module(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_to_string, m)?)?;
    m.add("ParsingError", py.get_type::<ParsingError>())?;
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

pub fn parse_query_string(query: &str) -> Result<Expression, ParsingError> {
    let mut parsed = parser::QLParser::parse(parser::Rule::start, query)?;
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
