
export type Number =
        // Variant Int64
    { kind: "Int64", data: bigint }

    |     // Variant Float
    { kind: "Float", data: number }

    | { kind: "RealNumber", data: RealNumber }
;


export class RealNumber {
    part0: number | undefined;
    part1: number | undefined;

    constructor(data: Partial<RealNumber>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            part_0: this.part0,
            part_1: this.part1,
        };
    }

    static fromJSON(json: any): RealNumber {
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

