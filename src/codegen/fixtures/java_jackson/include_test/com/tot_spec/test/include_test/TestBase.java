package com.tot_spec.test.include_test;
import lombok.Data;
import java.util.*;

@Data
public class TestBase {
    // use base's BaseId as the id
    private com.tot_spec.test.base.BaseId id;
    // use base_dup's BaseId as the id_2, this is just demo
    private com.tot_spec.test.base.BaseId id_2;
}