import { * as base } from "../include_base.yaml.ts";
import { * as base_dup } from "../include_base.yaml.ts";


export interface
TestBase {
    /// use base's BaseId as the id
    id: base.Id;
    /// use base_dup's BaseId as the id_2, this is just demo
    id_2: base_dup.Id;
    common: base.Common;
}

