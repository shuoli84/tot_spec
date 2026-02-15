
export type BaseId = bigint;


export class PageInfo {
    page!: number;
    pageSize!: number;

    constructor(data: Partial<PageInfo>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            page: this.page,
            page_size: this.pageSize,
        };
    }

    static fromJSON(json: any): PageInfo {
        return new PageInfo({
            page: json.page,
            pageSize: json.page_size,
        });
    }
}

export type PageInfoJSON = {
    page: number;
    page_size: number;
}

