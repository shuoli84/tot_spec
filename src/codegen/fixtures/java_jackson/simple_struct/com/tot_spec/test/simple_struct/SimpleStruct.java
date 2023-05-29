package com.tot_spec.test.simple_struct;
import lombok.*;
import java.util.*;

@Data
public class SimpleStruct {
    // bool value
    @com.fasterxml.jackson.annotation.JsonProperty("bool_value")
    private Boolean boolValue;
    // i8 value
    @com.fasterxml.jackson.annotation.JsonProperty("i8_value")
    private Integer i8Value;
    @com.fasterxml.jackson.annotation.JsonProperty("i64_value")
    private Integer i64Value;
    @com.fasterxml.jackson.annotation.JsonProperty("string_value")
    private String stringValue;
    @com.fasterxml.jackson.annotation.JsonProperty("bytes_value")
    private byte[] bytesValue;
    @com.fasterxml.jackson.annotation.JsonProperty("string_to_string")
    private Map<String, String> stringToString;
    // nested self
    private List<com.tot_spec.test.simple_struct.SimpleStruct> children;
}
