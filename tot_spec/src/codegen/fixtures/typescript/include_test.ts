import { * as base } from "../include_base.yaml.ts";
import { * as base_dup } from "../include_base.yaml.ts";


export class TestBase {
    /// use base's BaseId as the id
    id: base.Id;
    /// use base_dup's BaseId as the id_2, this is just demo
    id2: base_dup.Id;
    common: base.Common;

    constructor(data: Partial<TestBase>) {
        Object.assign(this, data);
    }

    toJSON(): any {
        return {
            id: this.id,
            id_2: this.id2,
            common: this.common.toJSON(),
        };
    }

    static fromJSON(json: {
        id: base.Id,
        id_2: base_dup.Id,
        common: base.Common,
    }): TestBase {
        return new TestBase({
            id: json.id,
            id2: json.id_2,
            common: Common.fromJSON(json.common),
        });
    }
}

export type TestBaseJSON = {
    id: base.Id;
    id_2: base_dup.Id;
    common: base.Common;
}

