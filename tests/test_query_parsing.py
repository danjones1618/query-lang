from typing import Protocol

import pytest
from django.db.models import Q
from pytest_benchmark.fixture import BenchmarkFixture

import rust_pest
from python_lark.parser import QueryParser as PythonLarkParser


class ParserFn(Protocol):
    def __call__(self, to_parse: str) -> Q: ...


@pytest.mark.parametrize(
    "parsing_fn",
    [
        pytest.param(rust_pest.parse_to_django_q, id="rust_pest"),
        pytest.param(PythonLarkParser().parse_to_django_q, id="python_lark"),
    ],
)
@pytest.mark.parametrize(
    ("lhs", "rhs"),
    [
        pytest.param('hi = "yes"', Q(hi__exact="yes"), id="simple-eq"),
        pytest.param('hi != "yes"', ~Q(hi__exact="yes"), id="simple-neq"),
        pytest.param('hi =~ "yes.*"', Q(hi__regex="yes.*"), id="simple-re"),
        pytest.param('hi !~ "yes.*"', ~Q(hi__regex="yes.*"), id="simple-nre"),
        pytest.param('hi   =   "yes"', Q(hi__exact="yes"), id="whitespace-eq"),
        pytest.param('hi   !=   "yes"', ~Q(hi__exact="yes"), id="whitespace-neq"),
        pytest.param('hi   =~   "yes.*"', Q(hi__regex="yes.*"), id="whitespace-re"),
        pytest.param('hi   !~   "yes.*"', ~Q(hi__regex="yes.*"), id="whitespace-nre"),
        pytest.param('hi = "yes" and yes = "no"', Q(hi__exact="yes") & Q(yes__exact="no"), id="and-eq"),
        pytest.param('hi != "yes" and yes = "no"', (~Q(hi__exact="yes")) & Q(yes__exact="no"), id="and-neq"),
        pytest.param('hi =~ "yes.*" and yes = "no"', Q(hi__regex="yes.*") & Q(yes__exact="no"), id="and-re"),
        pytest.param('hi !~ "yes.*" and yes = "no"', (~Q(hi__regex="yes.*")) & Q(yes__exact="no"), id="and-nre"),
        pytest.param('hi = "yes" or yes = "no"', Q(hi__exact="yes") | Q(yes__exact="no"), id="or-eq"),
        pytest.param('hi != "yes" or yes = "no"', (~Q(hi__exact="yes")) | Q(yes__exact="no"), id="or-neq"),
        pytest.param('hi =~ "yes.*" or yes = "no"', Q(hi__regex="yes.*") | Q(yes__exact="no"), id="or-re"),
        pytest.param('hi !~ "yes.*" or yes = "no"', (~Q(hi__regex="yes.*")) | Q(yes__exact="no"), id="or-nre"),
        pytest.param(
            """
            (hi = "yes" and location != "UK")
            or (yes !~ "okay" or no = "wow")
            or (a = "1" and b =    "3" and c         = "4")
            or a = "1"
            """,
            (Q(hi__exact="yes") & (~Q(location__exact="UK")))
            | ((~Q(yes__regex="okay")) | Q(no__exact="wow"))
            | (Q(a__exact="1") & Q(b__exact="3") & Q(c__exact="4"))
            | Q(a__exact="1"),
            id="multi-brackets",
        ),
    ],
)
def test_parsing(benchmark: BenchmarkFixture, parsing_fn: ParserFn, lhs: str, rhs: Q) -> None:
    assert benchmark(parsing_fn, lhs) == rhs
