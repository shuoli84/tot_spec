package com.tot_spec.example;
import lombok.*;
import java.util.*;

@com.fasterxml.jackson.annotation.JsonTypeInfo(use = com.fasterxml.jackson.annotation.JsonTypeInfo.Id.NAME, property = "type")
@com.fasterxml.jackson.annotation.JsonSubTypes({
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.I64.class, name = "I64"),
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.F64.class, name = "F64"),
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.RealNumber.class, name = "RealNumber"),
})
public abstract class Number {
    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public static class I64 extends Number {
        private Integer payload;
    }

    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public static class F64 extends Number {
        private Double payload;
    }

    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public static class RealNumber extends Number {
        private com.tot_spec.example.RealNumber payload;
    }
}
