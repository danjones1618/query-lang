import importlib.resources
from typing import Literal

from django.db.models import Q
from lark import Lark, Token, Transformer, v_args


@v_args(inline=True)
class QueryLangTranformer(Transformer[Token, Q]):
    @staticmethod
    def start(inner: Q) -> Q:
        return inner

    @staticmethod
    def expr(inner: Q) -> Q:
        return inner

    @staticmethod
    def or_expr_or_comparision(inner: Q) -> Q:
        return inner

    @staticmethod
    def and_expr_or_comparision(inner: Q) -> Q:
        return inner

    @staticmethod
    def bracket_expr(inner: Q) -> Q:
        return inner

    @staticmethod
    def and_expr(lhs: Q, rhs: Q) -> Q:
        return lhs & rhs

    @staticmethod
    def or_expr(lhs: Q, rhs: Q) -> Q:
        return lhs | rhs

    @staticmethod
    def comparison(field: str, op: Literal["=", "!=", "!~", "=~"], escaped_string: str) -> Q:
        match op:
            case "=" | "!=":
                operator = "exact"
            case "=~" | "!~":
                operator = "regex"

        result = Q(**{f"{field}__{operator}": escaped_string[1:-1]})

        if op in ("!=", "!~"):
            result = ~result

        return result


class QueryParser:
    def __init__(self) -> None:
        assert __package__ is not None
        grammar = importlib.resources.read_text(__package__, "grammar.lark")
        self.lark_parser = Lark(grammar=grammar, parser="lalr")
        self.transformer = QueryLangTranformer()

    def parse_to_django_q(self, to_parse: str) -> Q:
        parsed = self.lark_parser.parse(to_parse)
        return self.transformer.transform(parsed)
