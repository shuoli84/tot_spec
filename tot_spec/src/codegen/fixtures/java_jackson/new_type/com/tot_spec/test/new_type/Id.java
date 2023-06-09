package com.tot_spec.test.new_type;
import lombok.*;
import java.util.*;

// NewType to i64, and derive Ord macros
@lombok.EqualsAndHashCode
@Data
@Builder
@NoArgsConstructor
public class Id {
    private Integer value;

    @com.fasterxml.jackson.annotation.JsonCreator
    public Id(Integer value) {
        this.value = value;
    }

    @com.fasterxml.jackson.annotation.JsonValue
    public Integer getValue() {
        return value;
    }
}
