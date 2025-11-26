/**
 * @file This file contains the settings store for the application.
 * It uses `@tauri-store/svelte` to create persistent stores for user settings.
 */
import { RuneStore } from '@tauri-store/svelte';

export const DEFAULT_STATS = {
  totalDmg: true,
  dps: true,
  dmgPct: true,
  critRate: true,
  critDmgRate: true,
  luckyRate: false,
  luckyDmgRate: false,
  hits: false,
  hitsPerMinute: false,
};

export const DEFAULT_HISTORY_STATS = {
  totalDmg: true,
  dps: true,
  dmgPct: true,
  critRate: false,
  critDmgRate: false,
  luckyRate: false,
  luckyDmgRate: false,
  hits: false,
  hitsPerMinute: false,
};

export const DEFAULT_HISTORY_TANKED_STATS = {
  damageTaken: true,
  tankedPS: true,
  tankedPct: true,
  critTakenRate: false,
  critDmgRate: false,
  luckyRate: false,
  luckyDmgRate: false,
  hitsTaken: false,
  hitsPerMinute: false,
};

export const DEFAULT_HISTORY_HEAL_STATS = {
  healDealt: true,
  hps: true,
  healPct: true,
  critHealRate: false,
  critDmgRate: false,
  luckyRate: false,
  luckyDmgRate: false,
  hitsHeal: false,
  hitsPerMinute: false,
};

export type ShortcutSettingId = keyof typeof DEFAULT_SETTINGS.shortcuts;

const DEFAULT_GENERAL_SETTINGS = {
  showYourName: "Show Your Name",
  showOthersName: "Show Others' Name",
  showYourAbilityScore: true,
  showOthersAbilityScore: true,
  relativeToTopDPSPlayer: true,
  relativeToTopDPSSkill: true,
  relativeToTopHealPlayer: true,
  relativeToTopHealSkill: true,
  // Tanked specific relative-to-top toggles
  relativeToTopTankedPlayer: true,
  relativeToTopTankedSkill: true,
  shortenAbilityScore: true,
  shortenDps: true,
  shortenTps: true,
  bossOnlyDps: false,
  dungeonSegmentsEnabled: true,
  useDummyData: false,
};

export const DEFAULT_CLASS_COLORS: Record<string, string> = {
  "Stormblade": "#674598",
  "Frost Mage": "#4de3d1",
  "Wind Knight": "#0099c6",
  "Verdant Oracle": "#66aa00",
  "Heavy Guardian": "#b38915",
  "Marksman": "#ffee00",
  "Shield Knight": "#7b9aa2",
  "Beat Performer": "#ee2e48",
};

export const DEFAULT_FONT_SIZES = {
  xs: 10,    // Extra small - labels, hints (default 0.625rem = 10px)
  sm: 12,    // Small - secondary text (default 0.75rem = 12px)
  base: 14,  // Base - default text (default 0.875rem = 14px)
  lg: 16,    // Large - emphasis (default 1rem = 16px)
  xl: 20,    // Extra large - titles (default 1.25rem = 20px)
};

export const FONT_SIZE_LABELS: Record<string, string> = {
  xs: 'Extra Small',
  sm: 'Small',
  base: 'Base',
  lg: 'Large',
  xl: 'Extra Large',
};

const DEFAULT_SETTINGS = {
  accessibility: {
    // New theme setting; fallback to 'dark' if missing in existing persisted store
    theme: 'dark' as string,
    
    blur: false,
    transparency: false,
    transparentOpacityPercent: 2,
    condenseHeader: "all",
    // backward compatibility flag used in some places
    transparentMode: false,
    // Clickthrough mode
    clickthrough: false,
    // Custom class colors
    classColors: { ...DEFAULT_CLASS_COLORS },
    // Custom font sizes (in pixels)
    fontSizes: { ...DEFAULT_FONT_SIZES },
  },
  shortcuts: {
    showLiveMeter: "",
    hideLiveMeter: "",
    toggleLiveMeter: "",
    enableClickthrough: "",
    disableClickthrough: "",
    toggleClickthrough: "",
    resetEncounter: "",
    hardReset: "",
    toggleBossHp: "",
  },
  moduleSync: {
    enabled: false,
    apiKey: "",
    baseUrl: "https://api.bpsr.app/api/v1",
    autoSyncIntervalMinutes: 0,
    autoUpload: true,
  },
  live: {
    general: { ...DEFAULT_GENERAL_SETTINGS },
    dpsPlayers: { ...DEFAULT_STATS },
    dpsSkillBreakdown: { ...DEFAULT_STATS },
    healPlayers: { ...DEFAULT_STATS },
    healSkillBreakdown: { ...DEFAULT_STATS },
    tankedPlayers: { ...DEFAULT_STATS },
    tankedSkillBreakdown: { ...DEFAULT_STATS },
  },
  history: {
    general: { ...DEFAULT_GENERAL_SETTINGS },
    dpsPlayers: { ...DEFAULT_HISTORY_STATS },
    dpsSkillBreakdown: { ...DEFAULT_HISTORY_STATS },
    healPlayers: { ...DEFAULT_HISTORY_HEAL_STATS },
    healSkillBreakdown: { ...DEFAULT_HISTORY_STATS },
    tankedPlayers: { ...DEFAULT_HISTORY_TANKED_STATS },
    tankedSkillBreakdown: { ...DEFAULT_HISTORY_STATS },
  },
};

// We need flattened settings for every update to be able to auto-detect new changes
const RUNE_STORE_OPTIONS = { autoStart: true, saveOnChange: true };
export const SETTINGS = {
  accessibility: new RuneStore(
    'accessibility',
    DEFAULT_SETTINGS.accessibility,
    RUNE_STORE_OPTIONS
  ),
  shortcuts: new RuneStore(
    'shortcuts',
    DEFAULT_SETTINGS.shortcuts,
    RUNE_STORE_OPTIONS
  ),
  moduleSync: new RuneStore(
    'moduleSync',
    DEFAULT_SETTINGS.moduleSync,
    RUNE_STORE_OPTIONS
  ),
  live: {
    general: new RuneStore(
      'liveGeneral',
      DEFAULT_SETTINGS.live.general,
      RUNE_STORE_OPTIONS
    ),
    dps: {
      players: new RuneStore(
        'liveDpsPlayers',
        DEFAULT_SETTINGS.live.dpsPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'liveDpsSkillBreakdown',
        DEFAULT_SETTINGS.live.dpsSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
    heal: {
      players: new RuneStore(
        'liveHealPlayers',
        DEFAULT_SETTINGS.live.healPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'liveHealSkillBreakdown',
        DEFAULT_SETTINGS.live.healSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
    tanked: {
      players: new RuneStore(
        'liveTankedPlayers',
        DEFAULT_SETTINGS.live.tankedPlayers,
        RUNE_STORE_OPTIONS
      ),
      skills: new RuneStore(
        'liveTankedSkills',
        DEFAULT_SETTINGS.live.tankedSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
  },
  history: {
    general: new RuneStore(
      'historyGeneral',
      DEFAULT_SETTINGS.history.general,
      RUNE_STORE_OPTIONS
    ),
    dps: {
      players: new RuneStore(
        'historyDpsPlayers',
        DEFAULT_SETTINGS.history.dpsPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'historyDpsSkillBreakdown',
        DEFAULT_SETTINGS.history.dpsSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
    heal: {
      players: new RuneStore(
        'historyHealPlayers',
        DEFAULT_SETTINGS.history.healPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'historyHealSkillBreakdown',
        DEFAULT_SETTINGS.history.healSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
    tanked: {
      players: new RuneStore(
        'historyTankedPlayers',
        DEFAULT_SETTINGS.history.tankedPlayers,
        RUNE_STORE_OPTIONS
      ),
      skillBreakdown: new RuneStore(
        'historyTankedSkillBreakdown',
        DEFAULT_SETTINGS.history.tankedSkillBreakdown,
        RUNE_STORE_OPTIONS
      ),
    },
  },
  // persisted app metadata (tracks which app version the user last saw)
  appVersion: new RuneStore('appVersion', { value: '' }, RUNE_STORE_OPTIONS),
};

// Create flattened settings object for backwards compatibility
export const settings = {
  state: {
    accessibility: SETTINGS.accessibility.state,
    shortcuts: SETTINGS.shortcuts.state,
    moduleSync: SETTINGS.moduleSync.state,
    live: {
      general: SETTINGS.live.general.state,
      dps: {
        players: SETTINGS.live.dps.players.state,
        skillBreakdown: SETTINGS.live.dps.skillBreakdown.state,
      },
      heal: {
        players: SETTINGS.live.heal.players.state,
        skillBreakdown: SETTINGS.live.heal.skillBreakdown.state,
      },
      tanked: {
        players: SETTINGS.live.tanked.players.state,
        skills: SETTINGS.live.tanked.skills.state,
      },
    },
    appVersion: SETTINGS.appVersion.state,
    history: {
      general: SETTINGS.history.general.state,
      dps: {
        players: SETTINGS.history.dps.players.state,
        skillBreakdown: SETTINGS.history.dps.skillBreakdown.state,
      },
      heal: {
        players: SETTINGS.history.heal.players.state,
        skillBreakdown: SETTINGS.history.heal.skillBreakdown.state,
      },
      tanked: {
        players: SETTINGS.history.tanked.players.state,
        skillBreakdown: SETTINGS.history.tanked.skillBreakdown.state,
      },
    },
  },
};

// Accessibility helpers

// Available theme names (keep in sync with CSS classes defined in app.css)
export const AVAILABLE_THEMES = [
  'dark',
  'light',
  'pink',
  'green',
  'matcha',
  'rainbow'
];
