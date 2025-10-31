import type { ShortcutSettingId } from "$lib/settings-store";

export type BaseInputs = BaseInput[];

/** Common base for all settings */
export interface BaseInput {
  id: ShortcutSettingId;
  label: string;
  description?: string;
}
