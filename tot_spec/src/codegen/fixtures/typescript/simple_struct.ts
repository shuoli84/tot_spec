
// Example of simple struct definition
export class SimpleStruct {
    /// bool value
    boolValue!: boolean;
    /// i8 value
    i8Value!: number;
    i16Value: number | undefined;
    i32Value: number | undefined;
    i64Value: bigint | undefined;
    stringValue: string | undefined;
    bytesValue: Uint8Array | undefined;
    stringToString: Record<string, string> | undefined;
    /// nested self
    children: SimpleStruct[] | undefined;
    /// this field is required
    requiredStrValue!: string;

    constructor(data: Partial<SimpleStruct>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            bool_value: this.boolValue,
            i8_value: this.i8Value,
            i16_value: this.i16Value,
            i32_value: this.i32Value,
            i64_value: this.i64Value,
            string_value: this.stringValue,
            bytes_value: this.bytesValue,
            string_to_string: this.stringToString,
            children: this.children?.map((e) => e.toJSON()),
            required_str_value: this.requiredStrValue,
        };
    }

    static fromJSON(json: {
        bool_value: boolean,
        i8_value: number,
        i16_value: number | undefined,
        i32_value: number | undefined,
        i64_value: bigint | undefined,
        string_value: string | undefined,
        bytes_value: Uint8Array | undefined,
        string_to_string: Record<string, string> | undefined,
        children: SimpleStruct[] | undefined,
        required_str_value: string,
    }): SimpleStruct {
        return new SimpleStruct({
            boolValue: json.bool_value,
            i8Value: json.i8_value,
            i16Value: json.i16_value,
            i32Value: json.i32_value,
            i64Value: json.i64_value,
            stringValue: json.string_value,
            bytesValue: json.bytes_value,
            stringToString: json.string_to_string,
            children: json.children?.map((e: any) => SimpleStruct.fromJSON(e)),
            requiredStrValue: json.required_str_value,
        });
    }
}

export type SimpleStructJSON = {
    bool_value: boolean;
    i8_value: number;
    i16_value: number | undefined;
    i32_value: number | undefined;
    i64_value: bigint | undefined;
    string_value: string | undefined;
    bytes_value: Uint8Array | undefined;
    string_to_string: Record<string, string> | undefined;
    children: SimpleStruct[] | undefined;
    required_str_value: string;
}

