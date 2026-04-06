import * as TOML from "smol-toml";

export class PrettyPrinter {
    print(config: any): string {
        /* Requirement 7.6: Format AlloyConfig back into TOML */
        return TOML.stringify(config);
    }
}
