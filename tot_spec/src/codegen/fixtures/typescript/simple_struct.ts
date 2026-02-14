
// Example of simple struct definition
export interface
SimpleStruct {
    /// bool value
    boolValue: boolean;
    /// i8 value
    i8Value: number;
    i16Value: number | undefined;
    i32Value: number | undefined;
    i64Value: bigint | undefined;
    stringValue: string | undefined;
    bytesValue: Uint8Array | undefined;
    stringToString: Record<string, string> | undefined;
    /// nested self
    children: SimpleStruct[] | undefined;
    /// this field is required
    requiredStrValue: string;
}

