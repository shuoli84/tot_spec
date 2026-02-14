
export class SimpleStruct {
    boolValue: boolean;
    i8Value: number | undefined;
    i16Value: number | undefined;
    i32Value: number | undefined;
    i64Value: bigint | undefined;
    decimalValue: string | undefined;
    bigintValue: bigint | undefined;
    stringValue: string | undefined;
    bytesValue: Uint8Array | undefined;
    stringToString: Record<string, string> | undefined;
    keyValues: KeyValue | undefined;
    childrenContainer: Container | undefined;
    children: SimpleStruct[] | undefined;

    constructor(data: Partial<SimpleStruct>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            bool_value: this.boolValue,
            i8_value: this.i8Value,
            i16_value: this.i16Value,
            i32_value: this.i32Value,
            i64_value: this.i64Value,
            decimal_value: this.decimalValue,
            bigint_value: this.bigintValue,
            string_value: this.stringValue,
            bytes_value: this.bytesValue,
            string_to_string: this.stringToString,
            key_values: this.keyValues?.toJSON(),
            children_container: this.childrenContainer?.toJSON(),
            children: this.children?.map((e) => e.toJSON()),
        };
    }

    static fromJSON(json: {
        bool_value: boolean,
        i8_value: number | undefined,
        i16_value: number | undefined,
        i32_value: number | undefined,
        i64_value: bigint | undefined,
        decimal_value: string | undefined,
        bigint_value: bigint | undefined,
        string_value: string | undefined,
        bytes_value: Uint8Array | undefined,
        string_to_string: Record<string, string> | undefined,
        key_values: KeyValue | undefined,
        children_container: Container | undefined,
        children: SimpleStruct[] | undefined,
    }): SimpleStruct {
        return new SimpleStruct({
            boolValue: json.bool_value,
            i8Value: json.i8_value,
            i16Value: json.i16_value,
            i32Value: json.i32_value,
            i64Value: json.i64_value,
            decimalValue: json.decimal_value,
            bigintValue: json.bigint_value,
            stringValue: json.string_value,
            bytesValue: json.bytes_value,
            stringToString: json.string_to_string,
            keyValues: json.key_values ? KeyValue.fromJSON(json.key_values) : undefined,
            childrenContainer: json.children_container ? Container.fromJSON(json.children_container) : undefined,
            children: json.children?.map((e: any) => SimpleStruct.fromJSON(e)),
        });
    }
}

export type SimpleStructJSON = {
    bool_value: boolean;
    i8_value: number | undefined;
    i16_value: number | undefined;
    i32_value: number | undefined;
    i64_value: bigint | undefined;
    decimal_value: string | undefined;
    bigint_value: bigint | undefined;
    string_value: string | undefined;
    bytes_value: Uint8Array | undefined;
    string_to_string: Record<string, string> | undefined;
    key_values: KeyValue | undefined;
    children_container: Container | undefined;
    children: SimpleStruct[] | undefined;
}


export type KeyValue = Record<string, Uint8Array>;


export type Container = SimpleStruct[];


export class RealNumber {
    real: number | undefined;
    imagine: number | undefined;

    constructor(data: Partial<RealNumber>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            real: this.real,
            imagine: this.imagine,
        };
    }

    static fromJSON(json: {
        real: number | undefined,
        imagine: number | undefined,
    }): RealNumber {
        return new RealNumber({
            real: json.real,
            imagine: json.imagine,
        });
    }
}

export type RealNumberJSON = {
    real: number | undefined;
    imagine: number | undefined;
}


export type Number =
    { __type: "I64", payload: bigint }

    | { __type: "F64", payload: number }

    | { __type: "RealNumber", payload: RealNumber }
;


export interface BaseRequest {
    requestId: string | undefined;
}


export class AddRequest {
    requestId: string | undefined;
    numbers: Number[] | undefined;

    constructor(data: Partial<AddRequest>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            request_id: this.requestId,
            numbers: this.numbers?.map((e) => e.toJSON()),
        };
    }

    static fromJSON(json: {
        request_id: string | undefined,
        numbers: Number[] | undefined,
    }): AddRequest {
        return new AddRequest({
            requestId: json.request_id,
            numbers: json.numbers?.map((e: any) => Number.fromJSON(e)),
        });
    }
}

export type AddRequestJSON = {
    request_id: string | undefined;
    numbers: Number[] | undefined;
}


export class AddResponse {
    result: Number;

    constructor(data: Partial<AddResponse>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            result: this.result.toJSON(),
        };
    }

    static fromJSON(json: {
        result: Number,
    }): AddResponse {
        return new AddResponse({
            result: Number.fromJSON(json.result),
        });
    }
}

export type AddResponseJSON = {
    result: Number;
}


export class ResetRequest {
    requestId: string | undefined;

    constructor(data: Partial<ResetRequest>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            request_id: this.requestId,
        };
    }

    static fromJSON(json: {
        request_id: string | undefined,
    }): ResetRequest {
        return new ResetRequest({
            requestId: json.request_id,
        });
    }
}

export type ResetRequestJSON = {
    request_id: string | undefined;
}


export class ResetResponse {

    constructor(data: Partial<ResetResponse>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
        };
    }

    static fromJSON(json: {
    }): ResetResponse {
        return new ResetResponse({
        });
    }
}

export type ResetResponseJSON = {
}


export type ConstInteger = 1 | 2;


// used as swagger's spec_ad_method request
export class Request {
    value: bigint;

    constructor(data: Partial<Request>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    static fromJSON(json: {
        value: bigint,
    }): Request {
        return new Request({
            value: json.value,
        });
    }
}

export type RequestJSON = {
    value: bigint;
}


// used as swagger's spec_ad_method response
export class Response {
    value: bigint;

    constructor(data: Partial<Response>) {
        Object.assign(this, data);
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    static fromJSON(json: {
        value: bigint,
    }): Response {
        return new Response({
            value: json.value,
        });
    }
}

export type ResponseJSON = {
    value: bigint;
}

