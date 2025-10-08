export type BaseInputs = BaseInput[];

/** Common base for all settings */
export interface BaseInput {
  id: string;
  label: string;
  description?: string;
}
