
export interface Base {
    id: string;
}


export class Child {
    id!: string;
    name!: string;

    constructor(data: Partial<Child>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            id: this.id,
            name: this.name,
        };
    }

    static fromJSON(json: any): Child {
        return new Child({
            id: json.id,
            name: json.name,
        });
    }
}

export type ChildJSON = {
    id: string;
    name: string;
}

