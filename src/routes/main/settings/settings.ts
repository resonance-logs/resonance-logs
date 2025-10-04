/**
 * Type safety for settings schema
 */

/** Common base for all field definitions */
export interface BaseInputField<T> {
  type: string;
  label: string;
  description?: string;
  value: T;
}

/** Specific field types (extend as needed) */
export interface SwitchField extends BaseInputField<boolean> {
  type: "switch";
}
export interface ShortcutField extends BaseInputField<string> {
  type: "shortcut";
}

/** Union of all valid field types */
export type SettingField = SwitchField | ShortcutField;

/** A single category of settings */
export type SettingsCategory = Record<string, SettingField>;

/** The full settings schema */
export type SettingsSchema = Record<string, SettingsCategory>;
