"""
my_sdk - A Rust SDK with Python bindings
"""

from my_sdk._core import (
    add as _add,
    divide as _divide,
    get_external_uuid as _get_external_uuid,
)

__version__ = "0.1.0"
__all__ = ["add", "divide"]


def add(a: int, b: int) -> int:
    """
    Add two integers.

    Args:
        a: First integer
        b: Second integer

    Returns:
        The sum of a and b
    """
    return _add(a, b)


def divide(a: float, b: float) -> float:
    """
    Divide two numbers.

    Args:
        a: Numerator
        b: Denominator

    Returns:
        The result of a / b

    Raises:
        ValueError: If b is zero
    """
    return _divide(a, b)


def get_external_uuid() -> str:
    """
    Gets a random UUID via http call

    Returns:
        UUID string

    Raises:
        ValueError if http call fails
    """
    return _get_external_uuid()
