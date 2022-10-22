from dataclasses import dataclass
import abc
import typing


# Const def for i64
class Code(abc.ABC):
    # Everything is ok
    Ok: int = 0
    # Request is bad
    Error: int = 1
