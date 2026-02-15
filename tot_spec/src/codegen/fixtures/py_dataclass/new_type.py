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


@dataclass
class DictNewType:
    value: typing.Dict[str, bytes]

    def to_dict(self):
        result = {}
        for key, item in self.value.items():
            item_tmp = list(item)
            result[key] = item_tmp

        return result

    def from_dict(d):
        value_tmp = {}
        for key, item in d.items():
            item_tmp = bytes(item)
            value_tmp[key] = item_tmp

        return DictNewType(value_tmp)
