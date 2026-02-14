
// struct for decimal field
export class
TestDecimal {
    value: string | undefined;

    constructor(data: Partial<TestDecimal>) {
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

