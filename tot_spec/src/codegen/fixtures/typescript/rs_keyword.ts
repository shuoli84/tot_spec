
export class TestRustKeyword {
    fn: string | undefined;
    const_: number | undefined;

    constructor(data: Partial<TestRustKeyword>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            fn: this.fn,
            const: this.const_,
        };
    }

    static fromJSON(json: any): TestRustKeyword {
        return new TestRustKeyword({
            fn: json.fn,
            const_: json.const,
        });
    }
}

export type TestRustKeywordJSON = {
    fn: string | undefined;
    const: number | undefined;
}

