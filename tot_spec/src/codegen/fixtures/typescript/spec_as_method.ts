
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


export type Request = TestStruct;


export class Response {

    constructor(data: Partial<Response>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {};
    }

    static fromJSON(_json: any): Response {
        return new Response({});
    }
}

export type ResponseJSON = any;

