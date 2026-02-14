
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


export class
RealNumber {
    part0: number | undefined;
    part1: number | undefined;

    constructor(data: Partial<RealNumber>) {
        Object.assign(this, data);
    }

    toJSON(): {
        part_0: this.part0,
        part_1: this.part1,
    } {
        return {
            part_0,
            part_1,
        };
    }

    static fromJSON(json: {
        part_0: number | undefined,
        part_1: number | undefined,
    }): RealNumber {
        return new RealNumber({
            part0: json.part_0,
            part1: json.part_1,
        });
    }
}

export type RealNumberJSON = {
    part_0: number | undefined;
    part_1: number | undefined;
}

