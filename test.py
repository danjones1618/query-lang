import rust_pest as c
from rich import print

print(
    c.parse_to_django_q(
        '(hi = "yes" and location != "UK") or (yes !~ "okay" or no = "wow") or (a = "1" and b =    "3" and c         = "4") or a = "1"'
    )
)
