
// struct for bigint field
export class TestBigInt {
    value: bigint | undefined;

    constructor(data: Partial<TestBigInt>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            value: this.value,
        };
    }

    static fromJSON(json: any): TestBigInt {
        return new TestBigInt({
            value: json.value,
        });
    }
}

export type TestBigIntJSON = {
    value: bigint | undefined;
}

