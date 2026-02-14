
export type
Number =
        // Variant Int64
    { __type: "Int64", payload: bigint }
    |
    |     // Variant Float
    { __type: "Float", payload: number }
    |
    | { __type: "RealNumber", payload: RealNumber }
;


export interface
RealNumber {
    part_0: number | undefined;
    part_1: number | undefined;
}

