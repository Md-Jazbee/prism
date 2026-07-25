"""Minimal Python fixture for T1 golden conformance.

Covers: function defs, import, same-file call, unresolved call, class + extends.
"""
from os import path as ospath


def helper(x):
    return x + 1


class Base:
    pass


class Child(Base):
    def run(self):
        return helper(1)


def main():
    helper(2)
    missing_fn()
