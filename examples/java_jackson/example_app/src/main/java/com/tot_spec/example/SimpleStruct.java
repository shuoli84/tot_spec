package com.tot_spec.example;
import lombok.*;
import java.util.*;

@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class SimpleStruct {
    @com.fasterxml.jackson.annotation.JsonProperty("bool_value")
    private Boolean boolValue;
    @com.fasterxml.jackson.annotation.JsonProperty("i8_value")
    private Integer i8Value;
    @com.fasterxml.jackson.annotation.JsonProperty("i16_value")
    private Integer i16Value;
    @com.fasterxml.jackson.annotation.JsonProperty("i32_value")
    private Integer i32Value;
    @com.fasterxml.jackson.annotation.JsonProperty("i64_value")
    private Integer i64Value;
    @com.fasterxml.jackson.annotation.JsonProperty("decimal_value")
    private java.math.BigDecimal decimalValue;
    @com.fasterxml.jackson.annotation.JsonProperty("bigint_value")
    private java.math.BigInteger bigintValue;
    @com.fasterxml.jackson.annotation.JsonProperty("string_value")
    private String stringValue;
    @com.fasterxml.jackson.annotation.JsonProperty("bytes_value")
    private byte[] bytesValue;
    @com.fasterxml.jackson.annotation.JsonProperty("string_to_string")
    private Map<String, String> stringToString;
    @com.fasterxml.jackson.annotation.JsonProperty("key_values")
    private com.tot_spec.example.KeyValue keyValues;
    @com.fasterxml.jackson.annotation.JsonProperty("children_container")
    private com.tot_spec.example.Container childrenContainer;
    private List<com.tot_spec.example.SimpleStruct> children;
}
