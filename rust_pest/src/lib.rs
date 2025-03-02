use pest::error::Error as PestError;
use pest::iterators::Pair;
use pest::Parser;
use pyo3::exceptions::PyException;
use pyo3::types::IntoPyDict;

use pyo3::prelude::*;

#[pyclass(extends=PyException, module="query_lang._core", subclass)]
#[derive(Debug)]
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

mod python_wrappers {
    use pyo3::{prelude::*, types::PyDict};

    /// Wrapper around Djanog's Q.__init__ function.
    pub struct DjangoQueryInit<'py>(Bound<'py, PyAny>);

    /// Wrapper around Django's Q object
    pub struct DjangoQueryObject<'py>(Bound<'py, PyAny>);

    impl<'py> DjangoQueryInit<'py> {
        pub fn new<'pyinit>(py: Python<'py>) -> PyResult<DjangoQueryInit<'pyinit>>
        where
            'py: 'pyinit,
        {
            let query_model = py.import("django.db.models.sql.query")?;
            let q_init = query_model.getattr("Q")?;
            Ok(DjangoQueryInit(q_init))
        }

        pub fn init_q(&self, kwargs: &Bound<'py, PyDict>) -> PyResult<DjangoQueryObject<'py>> {
            Ok(DjangoQueryObject(self.0.call((), Some(&kwargs))?))
        }
    }

    impl<'py> DjangoQueryObject<'py> {
        pub fn invert(self) -> DjangoQueryObject<'py> {
            DjangoQueryObject(
                self.0
                    .call_method0("__invert__")
                    .expect("Q.__invert__ should not throw"),
            )
        }

        pub fn and(self, other: DjangoQueryObject<'py>) -> DjangoQueryObject<'py> {
            DjangoQueryObject(
                self.0
                    .call_method1("__and__", (other.0,))
                    .expect("Q.__and__ should not throw"),
            )
        }

        pub fn or(self, other: DjangoQueryObject<'py>) -> DjangoQueryObject<'py> {
            DjangoQueryObject(
                self.0
                    .call_method1("__or__", (other.0,))
                    .expect("Q.__and__ should not throw"),
            )
        }

        pub fn unwrap(self) -> Bound<'py, PyAny> {
            self.0
        }
    }

    impl<'py> ToString for DjangoQueryObject<'py> {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
}

use python_wrappers::{DjangoQueryInit, DjangoQueryObject};

fn expression_to_q<'py>(
    py: Python<'py>,
    q_init: &DjangoQueryInit<'py>,
    expression: &Expression,
) -> PyResult<DjangoQueryObject<'py>> {
    match expression {
        Expression::QueryItem(query_item) => query_item.into_q_object(py, q_init),
        Expression::CombinedExpression(combinator) => match combinator {
            Combinator::And { lhs, rhs } => {
                let lhs = expression_to_q(py, q_init, lhs)?;
                let rhs = expression_to_q(py, q_init, rhs)?;
                Ok(lhs.and(rhs))
            }
            Combinator::Or { lhs, rhs } => {
                let lhs = expression_to_q(py, q_init, lhs)?;
                let rhs = expression_to_q(py, q_init, rhs)?;
                Ok(lhs.or(rhs))
            }
        },
    }
}

#[pyfunction]
fn parse_to_django_q<'py>(py: Python<'py>, to_parse: &str) -> PyResult<Bound<'py, PyAny>> {
    let res = parse_query_string(&to_parse)?;
    let q_init = DjangoQueryInit::new(py)?;
    let result = expression_to_q(py, &q_init, &res)?;
    Ok(result.unwrap())
}

#[pymodule]
#[pyo3(name = "_core")]
fn query_lang_python_module(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(parse_to_django_q, module)?)?;
    module.add("ParsingError", py.get_type::<ParsingError>())?;
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
pub struct QueryItem<'a> {
    attribute: &'a str,
    match_type: MatchType,
    match_value: &'a str,
}

impl<'a> QueryItem<'a> {
    fn into_q_object<'py, 'pyinit>(
        &'a self,
        py: Python<'py>,
        q_init: &'pyinit DjangoQueryInit<'py>,
    ) -> PyResult<DjangoQueryObject<'py>>
    where
        'py: 'pyinit,
    {
        let type_string = match self.match_type {
            MatchType::Eq | MatchType::NotEq => "exact",
            MatchType::NotRegex | MatchType::Regex => "regex",
        };
        let should_invert = match self.match_type {
            MatchType::Eq | MatchType::Regex => false,
            MatchType::NotEq | MatchType::NotRegex => true,
        };
        let query_string = format!("{}__{}", self.attribute, type_string);

        let kwargs = vec![(query_string, self.match_value)].into_py_dict(py)?;
        let mut q_object = q_init.init_q(&kwargs)?;

        if should_invert {
            q_object = q_object.invert();
        }

        Ok(q_object)
    }
}

#[derive(Debug)]
pub enum Combinator<'a> {
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
                let attribute = rules.next().unwrap().into_inner().as_str();
                let match_type: MatchType = rules.next().unwrap().as_str().try_into().unwrap();
                let match_value = rules.next().unwrap().into_inner().as_str();
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
