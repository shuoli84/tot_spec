
export class TestStruct {
    value1!: string;

    constructor(data: Partial<TestStruct>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            value_1: this.value1,
        };
    }

    static fromJSON(json: any): TestStruct {
        return new TestStruct({
            value1: json.value_1,
        });
    }
}

export type TestStructJSON = {
    value_1: string;
}


export type TestRequest = TestStruct;


export class TestResponse {

    constructor(data: Partial<TestResponse>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {};
    }

    static fromJSON(_json: any): TestResponse {
        return new TestResponse({});
    }
}

export type TestResponseJSON = any;

