from example_spec import *

if __name__ == "__main__":
    int_val = Number_I64(123)
    float_val = Number_F64(23.0)

    print(int_val)
    print(isinstance(int_val, Number))
    print(float_val)
    print(isinstance(float_val, Number))

    number_list: typing.List[Number] = [int_val, float_val]
    print(number_list)

    add_request = AddRequest("foo_request_id", [1, 2, 3])
    print(add_request)

    reset_request = ResetRequest("foo_request_id")
    print(reset_request)

    simple_struct = SimpleStruct(
        True,
        1,
        i8_to_string={
            1: "hello_1",
            2: "hello_2",
        },
    )
    print(simple_struct)
    print(simple_struct.to_dict())

    import json

    print(json.dumps(simple_struct.to_dict()))
