
// struct for bigint field
export class
TestBigInt {
    value: bigint | undefined;

    constructor(data: Partial<TestBigInt>) {
        Object.assign(this, data);
    }

    toJSON(): {
        value: this.value,
    } {
        return {
            value,
        };
    }

    static fromJSON(json: {
        value: bigint | undefined,
    }): TestBigInt {
        return new TestBigInt({
            value: json.value,
        });
    }
}

export type TestBigIntJSON = {
    value: bigint | undefined;
}

