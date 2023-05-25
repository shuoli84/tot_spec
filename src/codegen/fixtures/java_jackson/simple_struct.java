package com.tot_spec.test.simple_struct;
import lombok.Data;
import java.util.*;

@Data
public class SimpleStruct {
    // bool value
    private Boolean bool_value;
    // i8 value
    private Integer i8_value;
    private Integer i64_value;
    private String string_value;
    private byte[] bytes_value;
    private Map<String, String> string_to_string;
    private KeyValue key_values;
    // nested self
    private List<SimpleStruct> children;
}