from dataclasses import dataclass
import abc
import typing


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
        json_value = None
        if item := d.get("json_value"):
            json_value = item
        return TestJsonStruct(
            json_value = json_value,
        )