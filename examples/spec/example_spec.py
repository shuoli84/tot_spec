from dataclasses import dataclass
import abc
import typing


# SimpleStruct
@dataclass
class SimpleStruct:
    bool_value: bool
    i8_value: int
    i64_value: typing.Optional[int] = None
    string_value: typing.Optional[str] = None
    bytes_value: typing.Optional[bytes] = None
    string_to_string: typing.Optional[typing.Dict[str, str]] = None
    key_values: typing.Optional["KeyValue"] = None
    children_container: typing.Optional["Container"] = None
    children: typing.Optional[typing.List["SimpleStruct"]] = None

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
            key_values_tmp = {}
            for key, item in self.key_values.items():
                item_tmp = list(item)
                key_values_tmp[key] = item_tmp
            
            result["key_values"] = key_values_tmp
    
        # children_container
        if self.children_container is None:
            result["children_container"] = None
        else:
            children_container_tmp = []
            for item in self.children_container:
                item_tmp = item.to_dict()
                children_container_tmp.append(item_tmp)
            
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
        bool_value = d["bool_value"]
        
        # i8_value
        i8_value = d["i8_value"]
        
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
            
        
        # key_values
        key_values = None
        if item := d.get("key_values"):
            key_values = {}
            for key, item in item.items():
                item_tmp = bytes(item)
                key_values[key] = item_tmp
            
        
        # children_container
        children_container = None
        if item := d.get("children_container"):
            children_container = []
            for item in item:
                item_tmp = SimpleStruct.from_dict(item)
                children_container.append(item_tmp)
            
        
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
            i64_value = i64_value,
            string_value = string_value,
            bytes_value = bytes_value,
            string_to_string = string_to_string,
            key_values = key_values,
            children_container = children_container,
            children = children,
        )
        
    

# KeyValue
KeyValue = typing.Type[typing.Dict[str, bytes]]

# Container
Container = typing.Type[typing.List["SimpleStruct"]]

# RealNumber
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
        real = d.get("real", None)
        
        # imagine
        imagine = d.get("imagine", None)
        return RealNumber(
            real = real,
            imagine = imagine,
        )
        
    

# Number
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
            payload_tmp = payload
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
    payload: "RealNumber"

    def to_dict(self):
        type_ = "RealNumber"
        payload_tmp = self.payload.to_dict()
        return {
            "type": type_,
            "payload": payload_tmp,
        }


# BaseRequest
class BaseRequest(abc.ABC):
    pass

    @staticmethod
    @abc.abstractmethod
    def from_dict(d): pass

    @abc.abstractmethod
    def to_dict(self): pass

# AddRequest
@dataclass
class AddRequest(BaseRequest):
    request_id: typing.Optional[str] = None
    numbers: typing.Optional[typing.List["Number"]] = None

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
        request_id = d.get("request_id", None)
        
        # numbers
        numbers = None
        if item := d.get("numbers"):
            numbers = []
            for item in item:
                item_tmp = Number.from_dict(item)
                numbers.append(item_tmp)
            
        return AddRequest(
            request_id = request_id,
            numbers = numbers,
        )
        
    

# ResetRequest
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
        request_id = d.get("request_id", None)
        return ResetRequest(
            request_id = request_id,
        )
        
    
