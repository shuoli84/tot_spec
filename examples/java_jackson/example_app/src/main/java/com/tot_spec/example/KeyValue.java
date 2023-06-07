package com.tot_spec.example;
import lombok.*;
import java.util.*;

public class KeyValue {
    private Map<String, byte[]> value;

    @com.fasterxml.jackson.annotation.JsonCreator
    public KeyValue(Map<String, byte[]> value) {
        this.value = value;
    }

    @com.fasterxml.jackson.annotation.JsonValue
    public Map<String, byte[]> getValue() {
        return value;
    }
}
