
// Test struct for json field
export class TestJsonStruct {
    jsonValue: unknown | undefined;

    constructor(data: Partial<TestJsonStruct>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            json_value: this.jsonValue,
        };
    }

    static fromJSON(json: any): TestJsonStruct {
        return new TestJsonStruct({
            jsonValue: json.json_value,
        });
    }
}

export type TestJsonStructJSON = {
    json_value: unknown | undefined;
}

