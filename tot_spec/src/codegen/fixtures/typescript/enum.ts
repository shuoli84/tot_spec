
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
    part0: number | undefined;
    part1: number | undefined;
}

