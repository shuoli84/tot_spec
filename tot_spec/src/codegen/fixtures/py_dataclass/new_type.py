# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


# NewType to i64, and derive Ord macros
@dataclass
class Id:
    value: int

    def to_dict(self):
        result = self.value
        return result

    def from_dict(d):
        value_tmp = int(d)
        return Id(value_tmp)

