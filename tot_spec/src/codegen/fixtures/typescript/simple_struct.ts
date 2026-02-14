
// Example of simple struct definition
export interface
SimpleStruct {
    /// bool value
    bool_value: boolean;
    /// i8 value
    i8_value: number;
    i16_value: number | undefined;
    i32_value: number | undefined;
    i64_value: bigint | undefined;
    string_value: string | undefined;
    bytes_value: Uint8Array | undefined;
    string_to_string: Record<string, string> | undefined;
    /// nested self
    children: SimpleStruct[] | undefined;
    /// this field is required
    required_str_value: string;
}

