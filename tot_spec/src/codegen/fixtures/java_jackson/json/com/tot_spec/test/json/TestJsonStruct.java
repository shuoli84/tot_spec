package com.tot_spec.test.json;
import lombok.*;
import java.util.*;

// Test struct for json field
@Data
@AllArgsConstructor
@NoArgsConstructor
public class TestJsonStruct {
    @com.fasterxml.jackson.annotation.JsonProperty("json_value")
    private com.fasterxml.jackson.databind.JsonNode jsonValue;
}
