import type {
    DPSRow
} from "$lib/bindings";

export class DPSRows {
    damageRows: DPSRow[];

    constructor(initialData: DPSRow[] = []) {
        this.damageRows = initialData;
    }
}
