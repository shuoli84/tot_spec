
export type Number =
        // Variant Int64
    { type: "Int64", payload: bigint }

    |     // Variant Float
    { type: "Float", payload: number }
;

