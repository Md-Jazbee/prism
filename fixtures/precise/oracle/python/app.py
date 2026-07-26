"""Caller — T1 leaves cross-file greet() unresolved; T2 PreciseIndex binds it."""
from lib import greet


def main():
    greet()
    missing_fn()
