# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


# Example of simple struct definition
@dataclass
class SimpleStruct:
    # bool value
    bool_value: bool
    # i8 value
    i8_value: int
    # this field is required
    required_str_value: str
    i64_value: typing.Optional[int] = None
    string_value: typing.Optional[str] = None
    bytes_value: typing.Optional[bytes] = None
    string_to_string: typing.Optional[typing.Dict[str, str]] = None
    # nested self
    children: typing.Optional[typing.List[SimpleStruct]] = None

    def to_dict(self):
        result = {}

        # bool_value
        result["bool_value"] = self.bool_value

        # i8_value
        result["i8_value"] = self.i8_value

        # required_str_value
        result["required_str_value"] = self.required_str_value

        # i64_value
        result["i64_value"] = self.i64_value

        # string_value
        result["string_value"] = self.string_value

        # bytes_value
        result["bytes_value"] = self.bytes_value

        # string_to_string
        if self.string_to_string is None:
            result["string_to_string"] = None
        else:
            string_to_string_tmp = {}
            for key, item in self.string_to_string.items():
                item_tmp = item
                string_to_string_tmp[key] = item_tmp

            result["string_to_string"] = string_to_string_tmp

        # children
        if self.children is None:
            result["children"] = None
        else:
            children_tmp = []
            for item in self.children:
                item_tmp = item.to_dict()
                children_tmp.append(item_tmp)

            result["children"] = children_tmp
        return result


    @staticmethod
    def from_dict(d):

        # bool_value
        bool_value = d["bool_value"]

        # i8_value
        i8_value = d["i8_value"]

        # required_str_value
        required_str_value = d["required_str_value"]

        # i64_value
        i64_value = d.get("i64_value", None)

        # string_value
        string_value = d.get("string_value", None)

        # bytes_value
        bytes_value = None
        if item := d.get("bytes_value"):
            bytes_value = bytes(item)

        # string_to_string
        string_to_string = None
        if item := d.get("string_to_string"):
            string_to_string = {}
            for key, item in item.items():
                item_tmp = item
                string_to_string[key] = item_tmp


        # children
        children = None
        if item := d.get("children"):
            children = []
            for item in item:
                item_tmp = SimpleStruct.from_dict(item)
                children.append(item_tmp)

        return SimpleStruct(
            bool_value = bool_value,
            i8_value = i8_value,
            required_str_value = required_str_value,
            i64_value = i64_value,
            string_value = string_value,
            bytes_value = bytes_value,
            string_to_string = string_to_string,
            children = children,
        )


