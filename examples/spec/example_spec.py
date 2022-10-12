from dataclasses import dataclass
import typing


# SimpleStruct
@dataclass
class SimpleStruct:
    bool_value: bool
    i8_value: int
    i64_value: typing.Optional[int] = None
    string_value: typing.Optional[str] = None
    bytes_value: typing.Optional[bytes] = None
    i8_to_string: typing.Optional[typing.Dict[int, str]] = None
    key_values: typing.Optional['KeyValue'] = None
    children: typing.Optional[typing.List['SimpleStruct']] = None

# KeyValue
KeyValue = typing.Type[typing.Dict[str, bytes]]

# Container
Container = typing.Type[typing.List['SimpleStruct']]

# Base
@dataclass
class Base:
    request_id: typing.Optional[str] = None

# Number
@dataclass
class Number:
    pass

# variant I64 for Number
@dataclass
class Number_I64(Number):
    payload: int

# variant F64 for Number
@dataclass
class Number_F64(Number):
    payload: float

# AddRequest
@dataclass
class AddRequest(Base):
    numbers: typing.Optional[typing.List['Number']] = None

# ResetRequest
@dataclass
class ResetRequest(Base):
    pass
