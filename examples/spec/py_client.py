from example_spec import *
import json

if __name__ == "__main__":
    int_val = Number_I64(123)
    assert isinstance(int_val, Number)
    assert json.dumps(int_val.to_dict()) == """{"type": "I64", "payload": 123}"""

    float_val = Number_F64(23.0)
    assert isinstance(float_val, Number)
    assert json.dumps(float_val.to_dict()) == """{"type": "F64", "payload": 23.0}"""

    real_num = Number_RealNumber(RealNumber(10, 100))
    assert isinstance(real_num, Number)
    assert (
        json.dumps(real_num.to_dict())
        == """{"type": "RealNumber", "payload": {"real": 10, "imagine": 100}}"""
    )

    # list of number
    number_list: typing.List[Number] = [int_val, float_val, real_num]
    number_list_json = [json.dumps(n.to_dict()) for n in number_list]
    number_list_back = [Number.from_dict(json.loads(s)) for s in number_list_json]
    print(number_list_back)

    add_request = AddRequest("foo_request_id", [1, 2, 3])
    print(add_request)

    reset_request = ResetRequest("foo_request_id")
    print(reset_request)

    simple_struct = SimpleStruct(
        True,
        1,
        string_to_string={
            "1": "hello_1",
            "2": "hello_2",
        },
        key_values={
            "foo_key": b"foo_value",
            "bar_key": b"bar_value",
        },
    )
    print(simple_struct)
    print(simple_struct.to_dict())

    dict_value = simple_struct.to_dict()
    print(f"json: {json.dumps(dict_value)}")
    dict_value_back = json.loads(json.dumps(dict_value))
    value_back = SimpleStruct.from_dict(dict_value_back)
    print(value_back)
    assert "1" in value_back.string_to_string
