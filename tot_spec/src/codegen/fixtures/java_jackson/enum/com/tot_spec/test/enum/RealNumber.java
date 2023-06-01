package com.tot_spec.test.enum;
import lombok.*;
import java.util.*;

@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class RealNumber {
    @com.fasterxml.jackson.annotation.JsonProperty("part_0")
    private Double part0;
    @com.fasterxml.jackson.annotation.JsonProperty("part_1")
    private Double part1;
}
