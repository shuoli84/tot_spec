package com.tot_spec.test.enum_custom_tag;
import lombok.*;
import java.util.*;

@com.fasterxml.jackson.annotation.JsonTypeInfo(use = com.fasterxml.jackson.annotation.JsonTypeInfo.Id.NAME, property = "kind")
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
        private Integer data;
    }

    @Data
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor
    public static class Float extends Number {
        private Double data;
    }

    @Data
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor
    public static class RealNumber extends Number {
        private com.tot_spec.test.enum_custom_tag.RealNumber data;
    }
}
