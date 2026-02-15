package com.tot_spec.test.new_type;
import lombok.*;
import java.util.*;

@Data
@Builder
@NoArgsConstructor
public class DictNewType {
    private Map<String, byte[]> value;

    @com.fasterxml.jackson.annotation.JsonCreator
    public DictNewType(Map<String, byte[]> value) {
        this.value = value;
    }

    @com.fasterxml.jackson.annotation.JsonValue
    public Map<String, byte[]> getValue() {
        return value;
    }
}
