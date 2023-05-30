package com.tot_spec.example;
import lombok.*;
import java.util.*;

@Data
@AllArgsConstructor
@NoArgsConstructor
public class AddRequest extends com.tot_spec.example.BaseRequest {
    private List<com.tot_spec.example.Number> numbers;
}
