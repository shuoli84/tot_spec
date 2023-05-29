package com.tot_spec.test.bigint;
import lombok.*;
import java.util.*;

public class TestClassNamespace {
    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public class Request {
        private com.tot_spec.test.bigint.TestClassNamespace.Operant left;
        private com.tot_spec.test.bigint.TestClassNamespace.Operant right;
    }

    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public class Response {
        private java.math.BigDecimal sum;
    }

    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public class Operant {
        private java.math.BigDecimal value;
    }
}
