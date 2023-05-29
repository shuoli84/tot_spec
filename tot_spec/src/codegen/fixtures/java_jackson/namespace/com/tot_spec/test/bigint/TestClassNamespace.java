package com.tot_spec.test.bigint;
import lombok.*;
import java.util.*;

public class TestClassNamespace {
    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public class Request {
        private java.math.BigInteger left;
        private java.math.BigInteger right;
    }
    @Data
    @AllArgsConstructor
    @NoArgsConstructor
    public class Response {
        private java.math.BigInteger sum;
    }
}
