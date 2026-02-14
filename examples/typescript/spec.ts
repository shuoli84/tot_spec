
export interface
SimpleStruct {
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
}


export type
KeyValue = Record<string, Uint8Array>;


export type
Container = SimpleStruct[];


export interface
RealNumber {
    real: number | undefined;
    imagine: number | undefined;
}


export type
Number =
    { __type: "I64", payload: bigint }
    |
    | { __type: "F64", payload: number }
    |
    | { __type: "RealNumber", payload: RealNumber }
;


export interface
BaseRequest {
    requestId: string | undefined;
}


export interface
AddRequest {
    requestId: string | undefined;
    numbers: Number[] | undefined;
}


export interface
AddResponse {
    result: Number;
}


export interface
ResetRequest {
    requestId: string | undefined;
}


export interface
ResetResponse {
}


export type
ConstInteger = 1 | 2;


// used as swagger's spec_ad_method request
export interface
Request {
    value: bigint;
}


// used as swagger's spec_ad_method response
export interface
Response {
    value: bigint;
}

