
export type Number =
        // Variant Int64
    { __type: "Int64", payload: bigint }

    |     // Variant Float
    { __type: "Float", payload: number }
;

