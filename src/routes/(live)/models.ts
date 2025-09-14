import type {
    DamageRow
} from "$lib/bindings";

export class DamageRows {
    damageRows: DamageRow[];

    constructor(initialData: DamageRow[] = []) {
        this.damageRows = initialData;
    }
}
