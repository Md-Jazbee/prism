"""Sample for T3 local slice fixtures.

Line numbers are 0-based in tree-sitter; criterion uses absolute file lines.
"""


def helper(x):
    a = x + 1
    return a


def bug(n):
    y = helper(n)
    if y > 10:
        y = y - 1
    return y
