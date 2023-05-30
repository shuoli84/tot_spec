package com.tot_spec.example;
import lombok.*;
import java.util.*;

@Data
public abstract class BaseRequest {
    @com.fasterxml.jackson.annotation.JsonProperty("request_id")
    private String requestId;
}
