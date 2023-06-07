# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


# Test struct for json field
@dataclass
class TestJsonStruct:
    json_value: typing.Optional[typing.Any] = None

    def to_dict(self):
        result = {}

        # json_value
        result["json_value"] = self.json_value
        return result


    @staticmethod
    def from_dict(d):

        # json_value
        json_value_tmp = None
        if item := d.get("json_value"):
            json_value_tmp = item
        return TestJsonStruct(
            json_value = json_value_tmp,
        )


