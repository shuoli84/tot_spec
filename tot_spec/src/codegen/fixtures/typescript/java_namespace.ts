
export class Request {
    left!: Operant;
    right!: Operant;

    constructor(data: Partial<Request>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            left: this.left.toJSON(),
            right: this.right.toJSON(),
        };
    }

    static fromJSON(json: any): Request {
        return new Request({
            left: Operant.fromJSON(json.left),
            right: Operant.fromJSON(json.right),
        });
    }
}

export type RequestJSON = {
    left: Operant;
    right: Operant;
}


export class Response {
    sum!: string;

    constructor(data: Partial<Response>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            sum: this.sum,
        };
    }

    static fromJSON(json: any): Response {
        return new Response({
            sum: json.sum,
        });
    }
}

export type ResponseJSON = {
    sum: string;
}


export class Operant {
    value: string | undefined;

    constructor(data: Partial<Operant>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            value: this.value,
        };
    }

    static fromJSON(json: any): Operant {
        return new Operant({
            value: json.value,
        });
    }
}

export type OperantJSON = {
    value: string | undefined;
}

