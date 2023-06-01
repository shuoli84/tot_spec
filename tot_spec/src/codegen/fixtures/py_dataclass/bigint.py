# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


# struct for bigint field
@dataclass
class TestBigInt:
    value: typing.Optional[int] = None

    def to_dict(self):
        result = {}

        # value
        if self.value is None:
            result["value"] = None
        else:
            value_tmp = str(self.value)
            result["value"] = value_tmp
        return result


    @staticmethod
    def from_dict(d):

        # value
        value = None
        if item := d.get("value"):
            value = int(item)
        return TestBigInt(
            value = value,
        )


