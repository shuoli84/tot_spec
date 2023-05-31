package com.tot_spec.test.include_test;
import lombok.*;
import java.util.*;

@Data
@Builder
@AllArgsConstructor
@NoArgsConstructor
public class TestBase {
    // use base's BaseId as the id
    private com.tot_spec.test.base.BaseId id;
    // use base_dup's BaseId as the id_2, this is just demo
    @com.fasterxml.jackson.annotation.JsonProperty("id_2")
    private com.tot_spec.test.base.BaseId id2;
}
