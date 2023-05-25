package com.tot_spec.test.enum;
import lombok.Data;
import java.util.*;

@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, property = "type")
@JsonSubTypes({
    @JsonSubTypes.Type(value = Number.Int64.class, name = "Int64"),
    @JsonSubTypes.Type(value = Number.Float.class, name = "Float"),
    @JsonSubTypes.Type(value = Number.RealNumber.class, name = "RealNumber"),
})
public abstract class Number {
    @Data
    public static class Int64 extends Number {
        private Integer payload;
    }

    @Data
    public static class Float extends Number {
        private Double payload;
    }

    @Data
    public static class RealNumber extends Number {
        private com.tot_spec.test.enum.RealNumber payload;
    }
}
