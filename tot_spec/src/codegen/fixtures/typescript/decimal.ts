
// struct for decimal field
export class TestDecimal {
    value: string | undefined;

    constructor(data: Partial<TestDecimal>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            value: this.value,
        };
    }

    static fromJSON(json: {
        value: string | undefined,
    }): TestDecimal {
        return new TestDecimal({
            value: json.value,
        });
    }
}

export type TestDecimalJSON = {
    value: string | undefined;
}

