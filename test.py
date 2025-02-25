import query_lang as c
from rich import print

try:
    print(c.parse_to_string('hi = "yes" and locatio(n = "UK"'))
except c.ParsingError as e:
    print(e)
