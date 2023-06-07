package com.tot_spec.example;
import lombok.*;
import java.util.*;

@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class RealNumber {
    private Double real;
    private Double imagine;
}
