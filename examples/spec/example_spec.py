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
    children_container: typing.Optional['Container'] = None
    children: typing.Optional[typing.List['SimpleStruct']] = None

    def to_dict(self):
        result = {}
    
        # bool_value
        result["bool_value"] = self.bool_value
    
        # i8_value
        result["i8_value"] = self.i8_value
    
        # i64_value
        result["i64_value"] = self.i64_value
    
        # string_value
        result["string_value"] = self.string_value
    
        # bytes_value
        result["bytes_value"] = self.bytes_value
    
        # i8_to_string
        i8_to_string_tmp = {}
        for key, item in (self.i8_to_string or {}).items():
            item_tmp = item
            i8_to_string_tmp[key] = item_tmp
        result["i8_to_string"] = i8_to_string_tmp
    
        # key_values
        key_values_tmp = {}
        for key, item in self.key_values.items():
            item_tmp = item
            key_values_tmp[key] = item_tmp
        
        result["key_values"] = key_values_tmp
    
        # children_container
        children_container_tmp = {}
        for item in self.children_container:
            item_tmp = item.to_dict()
            children_container_tmp.append(item_tmp)
        
        result["children_container"] = children_container_tmp
    
        # children
        children_tmp = []
        for item in self.children or []:
            item_tmp = item.to_dict()
            children_tmp.append(item_tmp)
        result["children"] = children_tmp
        return result
    

# KeyValue
KeyValue = typing.Type[typing.Dict[str, bytes]]

# Container
Container = typing.Type[typing.List['SimpleStruct']]

# Base
@dataclass
class Base:
    request_id: typing.Optional[str] = None

    def to_dict(self):
        result = {}
    
        # request_id
        result["request_id"] = self.request_id
        return result
    

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

    def to_dict(self):
        result = {}
    
        # numbers
        numbers_tmp = []
        for item in self.numbers or []:
            item_tmp = item.to_dict()
            numbers_tmp.append(item_tmp)
        result["numbers"] = numbers_tmp
        return result
    

# ResetRequest
@dataclass
class ResetRequest(Base):
    pass

    def to_dict(self):
        result = {}
        return result
    
