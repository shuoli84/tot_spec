package com.tot_spec.test.enum;
import lombok.*;
import java.util.*;

@com.fasterxml.jackson.annotation.JsonTypeInfo(use = com.fasterxml.jackson.annotation.JsonTypeInfo.Id.NAME, property = "type")
@com.fasterxml.jackson.annotation.JsonSubTypes({
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.Int64.class, name = "Int64"),
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.Float.class, name = "Float"),
    @com.fasterxml.jackson.annotation.JsonSubTypes.Type(value = Number.RealNumber.class, name = "RealNumber"),
})
public abstract class Number {
    @Data
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor
    public static class Int64 extends Number {
        private Integer payload;
    }

    @Data
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor
    public static class Float extends Number {
        private Double payload;
    }

    @Data
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor
    public static class RealNumber extends Number {
        private com.tot_spec.test.enum.RealNumber payload;
    }
}
