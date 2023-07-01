# import annotations to enable forward declaration
from __future__ import annotations
from dataclasses import dataclass
import abc
import typing
import decimal


@dataclass
class SimpleStruct:
    bool_value: bool
    i8_value: typing.Optional[int] = None
    i16_value: typing.Optional[int] = None
    i32_value: typing.Optional[int] = None
    i64_value: typing.Optional[int] = None
    decimal_value: typing.Optional[decimal.Decimal] = None
    bigint_value: typing.Optional[int] = None
    string_value: typing.Optional[str] = None
    bytes_value: typing.Optional[bytes] = None
    string_to_string: typing.Optional[typing.Dict[str, str]] = None
    key_values: typing.Optional[KeyValue] = None
    children_container: typing.Optional[Container] = None
    children: typing.Optional[typing.List[SimpleStruct]] = None

    def to_dict(self):
        result = {}

        # bool_value
        result["bool_value"] = self.bool_value

        # i8_value
        result["i8_value"] = self.i8_value

        # i16_value
        if self.i16_value is None:
            result["i16_value"] = None
        else:
            i16_value_tmp = self.i16_value
            result["i16_value"] = i16_value_tmp

        # i32_value
        if self.i32_value is None:
            result["i32_value"] = None
        else:
            i32_value_tmp = self.i32_value
            result["i32_value"] = i32_value_tmp

        # i64_value
        result["i64_value"] = self.i64_value

        # decimal_value
        if self.decimal_value is None:
            result["decimal_value"] = None
        else:
            decimal_value_tmp = str(self.decimal_value)
            result["decimal_value"] = decimal_value_tmp

        # bigint_value
        if self.bigint_value is None:
            result["bigint_value"] = None
        else:
            bigint_value_tmp = str(self.bigint_value)
            result["bigint_value"] = bigint_value_tmp

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

        # key_values
        if self.key_values is None:
            result["key_values"] = None
        else:
            key_values_tmp = self.key_values.to_dict()
            result["key_values"] = key_values_tmp

        # children_container
        if self.children_container is None:
            result["children_container"] = None
        else:
            children_container_tmp = self.children_container.to_dict()
            result["children_container"] = children_container_tmp

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
        bool_value_tmp = d["bool_value"]

        # i8_value
        i8_value_tmp = d.get("i8_value", None)

        # i16_value
        i16_value_tmp = None
        if item := d.get("i16_value"):
            i16_value_tmp = int(item)

        # i32_value
        i32_value_tmp = None
        if item := d.get("i32_value"):
            i32_value_tmp = int(item)

        # i64_value
        i64_value_tmp = d.get("i64_value", None)

        # decimal_value
        decimal_value_tmp = None
        if item := d.get("decimal_value"):
            decimal_value_tmp = decimal.Decimal(item)

        # bigint_value
        bigint_value_tmp = None
        if item := d.get("bigint_value"):
            bigint_value_tmp = int(item)

        # string_value
        string_value_tmp = d.get("string_value", None)

        # bytes_value
        bytes_value_tmp = None
        if item := d.get("bytes_value"):
            bytes_value = bytes(item)

        # string_to_string
        string_to_string_tmp = None
        if item := d.get("string_to_string"):
            string_to_string_tmp = {}
            for key, item in item.items():
                item_tmp = item
                string_to_string_tmp[key] = item_tmp


        # key_values
        key_values_tmp = None
        if item := d.get("key_values"):
            key_values_tmp = KeyValue.from_dict(item)

        # children_container
        children_container_tmp = None
        if item := d.get("children_container"):
            children_container_tmp = Container.from_dict(item)

        # children
        children_tmp = None
        if item := d.get("children"):
            children_tmp = []
            for item in item:
                item_tmp = SimpleStruct.from_dict(item)
                children_tmp.append(item_tmp)

        return SimpleStruct(
            bool_value = bool_value_tmp,
            i8_value = i8_value_tmp,
            i16_value = i16_value_tmp,
            i32_value = i32_value_tmp,
            i64_value = i64_value_tmp,
            decimal_value = decimal_value_tmp,
            bigint_value = bigint_value_tmp,
            string_value = string_value_tmp,
            bytes_value = bytes_value_tmp,
            string_to_string = string_to_string_tmp,
            key_values = key_values_tmp,
            children_container = children_container_tmp,
            children = children_tmp,
        )



@dataclass
class KeyValue:
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

        return KeyValue(value_tmp)


@dataclass
class Container:
    value: typing.List[SimpleStruct]

    def to_dict(self):
        result = []
for item in self.value:
    item_tmp = item.to_dict()
    result.append(item_tmp)

        return result

    def from_dict(d):
        value_tmp = []
for item in d:
    item_tmp = SimpleStruct.from_dict(item)
    value_tmp.append(item_tmp)

        return Container(value_tmp)


@dataclass
class RealNumber:
    real: typing.Optional[float] = None
    imagine: typing.Optional[float] = None

    def to_dict(self):
        result = {}

        # real
        result["real"] = self.real

        # imagine
        result["imagine"] = self.imagine
        return result


    @staticmethod
    def from_dict(d):

        # real
        real_tmp = d.get("real", None)

        # imagine
        imagine_tmp = d.get("imagine", None)
        return RealNumber(
            real = real_tmp,
            imagine = imagine_tmp,
        )



class Number(abc.ABC):
    pass

    @abc.abstractmethod
    def to_dict(self):
        pass

    @staticmethod
    def from_dict(d):
        type_ = d["type"]
        if type_ == "I64":
            payload = d["payload"]
            payload_tmp = int(payload)
            return Number_I64(payload=payload_tmp)
        elif type_ == "F64":
            payload = d["payload"]
            payload_tmp = payload
            return Number_F64(payload=payload_tmp)
        elif type_ == "RealNumber":
            payload = d["payload"]
            payload_tmp = RealNumber.from_dict(payload)
            return Number_RealNumber(payload=payload_tmp)
        else:
            raise ValueError(f"invalid type: {type_}")


# variant I64 for Number
@dataclass
class Number_I64(Number):
    payload: int

    def to_dict(self):
        type_ = "I64"
        payload_tmp = self.payload
        return {
            "type": type_,
            "payload": payload_tmp,
        }


# variant F64 for Number
@dataclass
class Number_F64(Number):
    payload: float

    def to_dict(self):
        type_ = "F64"
        payload_tmp = self.payload
        return {
            "type": type_,
            "payload": payload_tmp,
        }


# variant RealNumber for Number
@dataclass
class Number_RealNumber(Number):
    payload: RealNumber

    def to_dict(self):
        type_ = "RealNumber"
        payload_tmp = self.payload.to_dict()
        return {
            "type": type_,
            "payload": payload_tmp,
        }


class BaseRequest(abc.ABC):
    pass

    @staticmethod
    @abc.abstractmethod
    def from_dict(d): pass

    @abc.abstractmethod
    def to_dict(self): pass

@dataclass
class AddRequest(BaseRequest):
    request_id: typing.Optional[str] = None
    numbers: typing.Optional[typing.List[Number]] = None

    def to_dict(self):
        result = {}

        # request_id
        result["request_id"] = self.request_id

        # numbers
        if self.numbers is None:
            result["numbers"] = None
        else:
            numbers_tmp = []
            for item in self.numbers:
                item_tmp = item.to_dict()
                numbers_tmp.append(item_tmp)

            result["numbers"] = numbers_tmp
        return result


    @staticmethod
    def from_dict(d):

        # request_id
        request_id_tmp = d.get("request_id", None)

        # numbers
        numbers_tmp = None
        if item := d.get("numbers"):
            numbers_tmp = []
            for item in item:
                item_tmp = Number.from_dict(item)
                numbers_tmp.append(item_tmp)

        return AddRequest(
            request_id = request_id_tmp,
            numbers = numbers_tmp,
        )



@dataclass
class ResetRequest(BaseRequest):
    request_id: typing.Optional[str] = None

    def to_dict(self):
        result = {}

        # request_id
        result["request_id"] = self.request_id
        return result


    @staticmethod
    def from_dict(d):

        # request_id
        request_id_tmp = d.get("request_id", None)
        return ResetRequest(
            request_id = request_id_tmp,
        )



class ConstInteger(abc.ABC):
    Value1: int = 1
    Value2: int = 2
