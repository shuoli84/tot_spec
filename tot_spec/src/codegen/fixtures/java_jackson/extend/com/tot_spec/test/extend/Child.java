package com.tot_spec.test.extend;
import lombok.*;
import java.util.*;

@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class Child extends com.tot_spec.test.extend.Base {
    private String name;
}
