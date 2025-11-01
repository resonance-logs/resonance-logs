// Column configurations for history page tables
// This provides a clean way to render columns without repetitive if statements

export const historyDpsPlayerColumns = [
  { key: 'totalDmg', header: 'DMG', label: 'DMG', description: "Show player's total damage dealt", format: (v: number) => v.toLocaleString() },
  { key: 'dps', header: 'DPS', label: 'DPS', description: "Show player's damage per second", format: (v: number) => v.toFixed(1) },
  { key: 'dmgPct', header: 'D%', label: 'D%', description: "Show player's damage % contribution", format: (v: number) => v.toFixed(1) + '%' },
  { key: 'critRate', header: 'CR', label: 'CR', description: "Show player's critical rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'critDmgRate', header: 'CDMG', label: 'CDMG', description: "Show player's % damage that crit", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyRate', header: 'LR%', label: 'LR%', description: "Show player's lucky rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyDmgRate', header: 'LDMG%', label: 'LDMG%', description: "Show player's % damage that was lucky", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'hits', header: 'Hits', label: 'Hits', description: "Show player's total number of hits", format: (v: number) => v.toLocaleString() },
  { key: 'hitsPerMinute', header: 'HPM', label: 'HPM', description: "Show player's number of hits per minute", format: (v: number) => v.toFixed(1) },
] as const;

export const historyDpsSkillColumns = [
  { key: 'totalDmg', header: 'DMG', label: 'DMG', description: "Show skill's total damage dealt", format: (v: number) => v.toLocaleString() },
  { key: 'dps', header: 'DPS', label: 'DPS', description: "Show skill's damage per second", format: (v: number) => v.toFixed(1) },
  { key: 'dmgPct', header: 'D%', label: 'D%', description: "Show skill's damage % contribution", format: (v: number) => v.toFixed(1) + '%' },
  { key: 'critRate', header: 'CR', label: 'CR', description: "Show skill's critical rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'critDmgRate', header: 'CDMG', label: 'CDMG', description: "Show skill's % damage that crit", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyRate', header: 'LR%', label: 'LR%', description: "Show skill's lucky rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyDmgRate', header: 'LDMG%', label: 'LDMG%', description: "Show skill's % damage that was lucky", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'hits', header: 'Hits', label: 'Hits', description: "Show skill's total number of hits", format: (v: number) => v.toLocaleString() },
  { key: 'hitsPerMinute', header: 'HPM', label: 'HPM', description: "Show skill's number of hits per minute", format: (v: number) => v.toFixed(1) },
] as const;

export const historyHealPlayerColumns = [
  { key: 'healDealt', header: 'Heal', label: 'Heal', description: "Show player's total heal given", format: (v: number) => v.toLocaleString() },
  { key: 'hps', header: 'HPS', label: 'HPS', description: "Show player's heal per second", format: (v: number) => v.toFixed(1) },
  { key: 'healPct', header: 'H%', label: 'H%', description: "Show player's heal % contribution", format: (v: number) => v.toFixed(1) + '%' },
  { key: 'critHealRate', header: 'CR', label: 'CR', description: "Show player's heal critical rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'critDmgRate', header: 'CDMG', label: 'CDMG', description: "Show player's % heal that crit", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyRate', header: 'LR%', label: 'LR%', description: "Show player's heal lucky rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyDmgRate', header: 'LDMG%', label: 'LDMG%', description: "Show player's % heal that was lucky", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'hitsHeal', header: 'Hits', label: 'Hits', description: "Show player's total number of hits", format: (v: number) => v.toLocaleString() },
  { key: 'hitsPerMinute', header: 'HPM', label: 'HPM', description: "Show player's number of hits per minute", format: (v: number) => v.toFixed(1) },
] as const;

export const historyHealSkillColumns = [
  { key: 'totalDmg', header: 'Heal', label: 'Heal', description: "Show skill's total heal given", format: (v: number) => v.toLocaleString() },
  { key: 'dps', header: 'HPS', label: 'HPS', description: "Show skill's heal per second", format: (v: number) => v.toFixed(1) },
  { key: 'dmgPct', header: 'H%', label: 'H%', description: "Show skill's heal % contribution", format: (v: number) => v.toFixed(1) + '%' },
  { key: 'critRate', header: 'CR', label: 'CR', description: "Show skill's critical rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'critDmgRate', header: 'CDMG', label: 'CDMG', description: "Show skill's % heal that crit", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyRate', header: 'LR%', label: 'LR%', description: "Show skill's heal lucky rate", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'luckyDmgRate', header: 'LDMG%', label: 'LDMG%', description: "Show skill's % heal that was lucky", format: (v: number) => (v * 100).toFixed(1) + '%' },
  { key: 'hits', header: 'Hits', label: 'Hits', description: "Show skill's total number of hits", format: (v: number) => v.toLocaleString() },
  { key: 'hitsPerMinute', header: 'HPM', label: 'HPM', description: "Show skill's number of hits per minute", format: (v: number) => v.toFixed(1) },
] as const;
