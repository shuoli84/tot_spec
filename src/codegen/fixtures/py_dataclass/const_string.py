from dataclasses import dataclass
import abc
import typing


# Const def for string
class Reason(abc.ABC):
    # Everything is ok
    Ok: str = "ok"
    # Request is bad
    Error: str = "error"
