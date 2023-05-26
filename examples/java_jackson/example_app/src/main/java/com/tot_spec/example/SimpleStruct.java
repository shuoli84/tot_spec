package com.tot_spec.example;
import lombok.*;
import java.util.*;

@Data
public class SimpleStruct {
    private Boolean boolValue;
    private Integer i8Value;
    private Integer i64Value;
    private String stringValue;
    private byte[] bytesValue;
    private Map<String, String> stringToString;
    private com.tot_spec.example.KeyValue keyValues;
    private com.tot_spec.example.Container childrenContainer;
    private List<com.tot_spec.example.SimpleStruct> children;
}
