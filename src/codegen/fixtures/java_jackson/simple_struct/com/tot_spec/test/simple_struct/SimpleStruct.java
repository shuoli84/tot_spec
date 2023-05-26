package com.tot_spec.test.simple_struct;
import lombok.*;
import java.util.*;

@Data
public class SimpleStruct {
    // bool value
    private Boolean boolValue;
    // i8 value
    private Integer i8Value;
    private Integer i64Value;
    private String stringValue;
    private byte[] bytesValue;
    private Map<String, String> stringToString;
    // nested self
    private List<com.tot_spec.test.simple_struct.SimpleStruct> children;
}
