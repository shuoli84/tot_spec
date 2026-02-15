
export class TestStruct {
    /// camel case to test ignore style
    valueString!: string;

    constructor(data: Partial<TestStruct>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            valueString: this.valueString,
        };
    }

    static fromJSON(json: any): TestStruct {
        return new TestStruct({
            valueString: json.valueString,
        });
    }
}

export type TestStructJSON = {
    valueString: string;
}

