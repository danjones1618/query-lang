__all__ = [
    "ParsingError",
    "parse_to_string",
]

class ParsingError(Exception): ...

def parse_to_string(to_parse: str) -> str: ...
