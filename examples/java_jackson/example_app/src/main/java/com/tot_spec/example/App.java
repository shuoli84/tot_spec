package com.tot_spec.example;

import java.io.IOException;

import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Hello world!
 *
 */
public class App {
    public static void main(String[] args) throws IOException {
        var number = new Number.I64(1);
        System.out.println(number);

        var simple = new SimpleStruct();
        simple.setBoolValue(true);
        simple.setBytesValue("hello world".getBytes());

        var object_mapper = new ObjectMapper();
        var json_str = object_mapper.writeValueAsString(simple);
        System.out.println(json_str);

        var simple2 = object_mapper.readValue(json_str, SimpleStruct.class);
        System.out.println(simple2);
    }
}