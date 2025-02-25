__all__ = [
    "ParsingError",
    "parse_to_string",
]

class ParsingError(Exception):
    message: str

    def __init__(self, message: str) -> None: ...

def parse_to_string(to_parse: str) -> str: ...
