# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


class Number(abc.ABC):
    pass

    @abc.abstractmethod
    def to_dict(self):
        pass

    @staticmethod
    def from_dict(d):
        type_ = d["kind"]
        if type_ == "Int64":
            data = d["data"]
            payload_tmp = int(data)
            return Number_Int64(data=payload_tmp)
        elif type_ == "Float":
            data = d["data"]
            payload_tmp = data
            return Number_Float(data=payload_tmp)
        elif type_ == "RealNumber":
            data = d["data"]
            payload_tmp = RealNumber.from_dict(data)
            return Number_RealNumber(data=payload_tmp)
        else:
            raise ValueError(f"invalid type: {type_}")


# variant Int64 for Number
@dataclass
class Number_Int64(Number):
    data: int

    def to_dict(self):
        type_ = "Int64"
        payload_tmp = self.data
        return {
            "kind": type_,
            "data": payload_tmp,
        }


# variant Float for Number
@dataclass
class Number_Float(Number):
    data: float

    def to_dict(self):
        type_ = "Float"
        payload_tmp = self.data
        return {
            "kind": type_,
            "data": payload_tmp,
        }


# variant RealNumber for Number
@dataclass
class Number_RealNumber(Number):
    data: RealNumber

    def to_dict(self):
        type_ = "RealNumber"
        payload_tmp = self.data.to_dict()
        return {
            "kind": type_,
            "data": payload_tmp,
        }


@dataclass
class RealNumber:
    part_0: typing.Optional[float] = None
    part_1: typing.Optional[float] = None

    def to_dict(self):
        result = {}

        # part_0
        result["part_0"] = self.part_0

        # part_1
        result["part_1"] = self.part_1
        return result


    @staticmethod
    def from_dict(d):

        # part_0
        part_0_tmp = d.get("part_0", None)

        # part_1
        part_1_tmp = d.get("part_1", None)
        return RealNumber(
            part_0 = part_0_tmp,
            part_1 = part_1_tmp,
        )


