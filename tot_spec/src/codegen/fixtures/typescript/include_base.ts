
export type Id = bigint;


export class Common {
    id!: bigint;

    constructor(data: Partial<Common>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            id: this.id,
        };
    }

    static fromJSON(json: any): Common {
        return new Common({
            id: json.id,
        });
    }
}

export type CommonJSON = {
    id: bigint;
}

