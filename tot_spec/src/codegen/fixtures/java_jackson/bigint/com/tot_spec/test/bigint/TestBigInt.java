package com.tot_spec.test.bigint;
import lombok.*;
import java.util.*;

// struct for bigint field
@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class TestBigInt {
    private java.math.BigInteger value;
}
