import query_lang._core as c
from rich import print

print(c.parse_to_string('hi = "yes" and location = "UK"'))
