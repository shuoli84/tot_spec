package com.tot_spec.example;
import lombok.*;
import java.util.*;

public class Container {
    private List<com.tot_spec.example.SimpleStruct> value;

    @com.fasterxml.jackson.annotation.JsonCreator
    public Container(List<com.tot_spec.example.SimpleStruct> value) {
        this.value = value;
    }

    @com.fasterxml.jackson.annotation.JsonValue
    public List<com.tot_spec.example.SimpleStruct> get_value() {
        return value;
    }
}
