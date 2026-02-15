
// Empty struct with no fields
export class EmptyStruct {

    constructor(data: Partial<EmptyStruct>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {};
    }

    static fromJSON(_json: any): EmptyStruct {
        return new EmptyStruct({});
    }
}

export type EmptyStructJSON = {};

