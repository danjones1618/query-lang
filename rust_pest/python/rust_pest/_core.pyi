from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from django.db.models.query import Q

__all__ = [
    "ParsingError",
    "parse_to_django_q",
]

class ParsingError(Exception):
    message: str

    def __init__(self, message: str) -> None: ...

def parse_to_django_q(to_parse: str) -> "Q": ...
