/**
 * @file This file contains the settings store for the application.
 * It uses `@tauri-store/svelte` to create persistent stores for user settings.
 */
import { RuneStore } from '@tauri-store/svelte';

export const DEFAULT_STATS = {
  totalDmg: true,
  dps: true,
  tdps: false,
  bossDmg: true,
  bossDps: true,
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
  tdps: false,
  bossDmg: true,
  bossDps: true,
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

// Default column order for live tables (keys from column-data.ts)
export const DEFAULT_DPS_PLAYER_COLUMN_ORDER = ['totalDmg', 'dps', 'tdps', 'bossDmg', 'bossDps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];
export const DEFAULT_DPS_SKILL_COLUMN_ORDER = ['totalDmg', 'dps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];
export const DEFAULT_HEAL_PLAYER_COLUMN_ORDER = ['totalDmg', 'dps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];
export const DEFAULT_HEAL_SKILL_COLUMN_ORDER = ['totalDmg', 'dps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];
export const DEFAULT_TANKED_PLAYER_COLUMN_ORDER = ['totalDmg', 'dps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];
export const DEFAULT_TANKED_SKILL_COLUMN_ORDER = ['totalDmg', 'dps', 'dmgPct', 'critRate', 'critDmgRate', 'luckyRate', 'luckyDmgRate', 'hits', 'hitsPerMinute'];

// Default sort settings for live tables
export const DEFAULT_LIVE_SORT_SETTINGS = {
  dpsPlayers: { sortKey: 'totalDmg', sortDesc: true },
  dpsSkills: { sortKey: 'totalDmg', sortDesc: true },
  healPlayers: { sortKey: 'totalDmg', sortDesc: true },
  healSkills: { sortKey: 'totalDmg', sortDesc: true },
  tankedPlayers: { sortKey: 'totalDmg', sortDesc: true },
  tankedSkills: { sortKey: 'totalDmg', sortDesc: true },
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
  dungeonSegmentsEnabled: false,
  wipeDetection: true,
  useDummyData: false,
  eventUpdateRateMs: 200,
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

export const CLASS_SPEC_MAP: Record<string, string> = {
  "Iaido": "Stormblade", "Moonstrike": "Stormblade",
  "Icicle": "Frost Mage", "Frostbeam": "Frost Mage",
  "Vanguard": "Wind Knight", "Skyward": "Wind Knight",
  "Smite": "Verdant Oracle", "Lifebind": "Verdant Oracle",
  "Earthfort": "Heavy Guardian", "Block": "Heavy Guardian",
  "Wildpack": "Marksman", "Falconry": "Marksman",
  "Recovery": "Shield Knight", "Shield": "Shield Knight",
  "Dissonance": "Beat Performer", "Concerto": "Beat Performer",
};

export const CLASS_SPEC_NAMES = Object.keys(CLASS_SPEC_MAP);

export const DEFAULT_CLASS_SPEC_COLORS: Record<string, string> = {
  // Stormblade
  "Iaido": "#9b6cf0", "Moonstrike": "#4a2f80",
  // Frost Mage
  "Icicle": "#8ff7ee", "Frostbeam": "#2fbfb3",
  // Wind Knight
  "Vanguard": "#4ddff6", "Skyward": "#006b8f",
  // Verdant Oracle
  "Smite": "#b9f36e", "Lifebind": "#3b6d00",
  // Heavy Guardian
  "Earthfort": "#e6c25a", "Block": "#7b5b08",
  // Marksman
  "Wildpack": "#fff9a6", "Falconry": "#cab400",
  // Shield Knight
  "Recovery": "#b6d1d6", "Shield": "#4f6b70",
  // Beat Performer
  "Dissonance": "#ff7b94", "Concerto": "#9f1322",
};

export const DEFAULT_FONT_SIZES = {
  xs: 10,    // Extra small - labels, hints (default 0.625rem = 10px)
  sm: 12,    // Small - secondary text (default 0.75rem = 12px)
  base: 14,  // Base - default text (default 0.875rem = 14px)
  lg: 16,    // Large - emphasis (default 1rem = 16px)
  xl: 20,    // Extra large - titles (default 1.25rem = 20px)
};

// Live table customization defaults
export const DEFAULT_LIVE_TABLE_SETTINGS = {
  // Player row settings
  playerRowHeight: 28,
  playerFontSize: 13,
  playerIconSize: 20,



  // Table header settings
  showTableHeader: true,
  tableHeaderHeight: 24,
  tableHeaderFontSize: 11,
  tableHeaderTextColor: "#a1a1aa",

  // Abbreviated numbers (K, M, %)
  abbreviatedFontSize: 10,



  // Skill row settings (separate from player rows)
  skillRowHeight: 24,
  skillFontSize: 12,
  skillIconSize: 18,


  skillShowHeader: true,
  skillHeaderHeight: 22,
  skillHeaderFontSize: 10,
  skillHeaderTextColor: "#a1a1aa",
  skillAbbreviatedFontSize: 9,


  // Skill-specific row glow / highlight customization (separate from player rows)
  skillRowGlowMode: 'gradient-underline' as 'gradient-underline' | 'gradient' | 'solid',
  skillRowGlowOpacity: 0.15,
  skillRowBorderRadius: 0,
  // Row glow / highlight customization
  // modes: 'gradient-underline' (gradient + neon underline), 'gradient' (gradient only), 'solid' (solid color fill)
  rowGlowMode: 'gradient-underline' as 'gradient-underline' | 'gradient' | 'solid',
  // opacity applied to the fill (0-1)
  rowGlowOpacity: 0.15,
  // border height in pixels for the neon underline effect
  rowGlowBorderHeight: 2,
  // box-shadow spread/blur for the neon border
  rowGlowSpread: 8,
  // Note: glow always uses the detected class/spec color.
  // Row border customization
  rowBorderRadius: 0,
};

// (Header preset constants removed - header defaults inlined into DEFAULT_SETTINGS)

export const FONT_SIZE_LABELS: Record<string, string> = {
  xs: 'Extra Small',
  sm: 'Small',
  base: 'Base',
  lg: 'Large',
  xl: 'Extra Large',
};

// Default custom theme colors (based on dark theme)
export const DEFAULT_CUSTOM_THEME_COLORS: Record<string, string> = {
  backgroundMain: 'rgba(33, 33, 33, 1)',
  backgroundLive: 'rgba(33, 33, 33, 1)',
  foreground: 'rgba(226, 226, 226, 1)',
  surface: 'rgba(41, 41, 41, 1)',
  surfaceForeground: 'rgba(226, 226, 226, 1)',
  primary: 'rgba(166, 166, 166, 1)',
  primaryForeground: 'rgba(33, 33, 33, 1)',
  secondary: 'rgba(64, 64, 64, 1)',
  secondaryForeground: 'rgba(226, 226, 226, 1)',
  muted: 'rgba(56, 56, 56, 1)',
  mutedForeground: 'rgba(138, 138, 138, 1)',
  accent: 'rgba(82, 82, 82, 1)',
  accentForeground: 'rgba(226, 226, 226, 1)',
  destructive: 'rgba(220, 80, 80, 1)',
  destructiveForeground: 'rgba(255, 255, 255, 1)',
  border: 'rgba(74, 74, 74, 1)',
  input: 'rgba(64, 64, 64, 1)',
  tooltipBg: 'rgba(33, 33, 33, 0.92)',
  tooltipBorder: 'rgba(74, 74, 74, 0.55)',
  tooltipFg: 'rgba(226, 226, 226, 1)',
  tableTextColor: '#ffffff',
  tableAbbreviatedColor: '#71717a',
};

// Labels for custom theme color variables
export const CUSTOM_THEME_COLOR_LABELS: Record<string, { label: string; description: string; category: string }> = {
  backgroundMain: { label: 'Background (Main)', description: 'Background color for main window', category: 'Base' },
  backgroundLive: { label: 'Background (Live)', description: 'Background color for live meter', category: 'Base' },
  foreground: { label: 'Foreground', description: 'Main text color', category: 'Base' },
  surface: { label: 'Surface', description: 'Background for cards, popovers, and panels', category: 'Surfaces' },
  surfaceForeground: { label: 'Surface Text', description: 'Text on surfaces', category: 'Surfaces' },
  primary: { label: 'Primary', description: 'Primary accent color', category: 'Accents' },
  primaryForeground: { label: 'Primary Text', description: 'Text on primary elements', category: 'Accents' },
  secondary: { label: 'Secondary', description: 'Secondary accent color', category: 'Accents' },
  secondaryForeground: { label: 'Secondary Text', description: 'Text on secondary elements', category: 'Accents' },
  muted: { label: 'Muted', description: 'Muted/subtle background', category: 'Utility' },
  mutedForeground: { label: 'Muted Text', description: 'Subdued text color', category: 'Utility' },
  accent: { label: 'Accent', description: 'Highlight accent color', category: 'Accents' },
  accentForeground: { label: 'Accent Text', description: 'Text on accent elements', category: 'Accents' },
  destructive: { label: 'Destructive', description: 'Error/danger color', category: 'Utility' },
  destructiveForeground: { label: 'Destructive Text', description: 'Text on destructive elements', category: 'Utility' },
  border: { label: 'Border', description: 'Border color', category: 'Utility' },
  input: { label: 'Input', description: 'Input field background', category: 'Utility' },
  tableTextColor: { label: 'Table Text', description: 'Color for text in live tables', category: 'Tables' },
  tableAbbreviatedColor: { label: 'Suffix Color', description: 'Color of K, M, % suffixes in tables', category: 'Tables' },
  tooltipBg: { label: 'Tooltip Background', description: 'Background color for tooltips', category: 'Tooltip' },
  tooltipBorder: { label: 'Tooltip Border', description: 'Border color for tooltips', category: 'Tooltip' },
  tooltipFg: { label: 'Tooltip Text', description: 'Text color for tooltips', category: 'Tooltip' },
};

const DEFAULT_SETTINGS = {
  accessibility: {
    blur: false,
    clickthrough: false,
    classColors: { ...DEFAULT_CLASS_COLORS },
    useClassSpecColors: false,
    classSpecColors: { ...DEFAULT_CLASS_SPEC_COLORS },
    fontSizes: { ...DEFAULT_FONT_SIZES },
    customThemeColors: { ...DEFAULT_CUSTOM_THEME_COLORS },
    // Background image settings
    backgroundImage: '' as string,
    backgroundImageEnabled: false,
    backgroundImageMode: 'cover' as 'cover' | 'contain',
    backgroundImageContainColor: 'rgba(0, 0, 0, 1)',
    // Custom font settings
    customFontSansEnabled: false,
    customFontSansUrl: '' as string,
    customFontSansName: '' as string,
    customFontMonoEnabled: false,
    customFontMonoUrl: '' as string,
    customFontMonoName: '' as string,
  },
  shortcuts: {
    showLiveMeter: "",
    hideLiveMeter: "",
    toggleLiveMeter: "",
    enableClickthrough: "",
    disableClickthrough: "",
    toggleClickthrough: "",
    resetEncounter: "",
      togglePauseEncounter: "",
    hardReset: "",
    toggleBossHp: "",
  },
  moduleSync: {
    enabled: false,
    apiKey: "",
    baseUrl: "https://api.bpsr.app/api/v1",
    autoSyncIntervalMinutes: 0,
    autoUpload: true,
    marketUpload: true,
  },
  live: {
    general: { ...DEFAULT_GENERAL_SETTINGS },
    dpsPlayers: { ...DEFAULT_STATS },
    dpsSkillBreakdown: { ...DEFAULT_STATS },
    healPlayers: { ...DEFAULT_STATS },
    healSkillBreakdown: { ...DEFAULT_STATS },
    tankedPlayers: { ...DEFAULT_STATS },
    tankedSkillBreakdown: { ...DEFAULT_STATS },
    tableCustomization: { ...DEFAULT_LIVE_TABLE_SETTINGS },
    headerCustomization: {
      windowPadding: 12,
      headerPadding: 8,
      showTimer: true,
      showSceneName: true,
      showSegmentInfo: true,
      showResetButton: true,
      showPauseButton: true,
      showBossOnlyButton: true,
      showSettingsButton: true,
      showMinimizeButton: true,
      showTotalDamage: true,
      showTotalDps: true,
      showBossHealth: true,
      showNavigationTabs: true,
      timerLabelFontSize: 12,
      timerFontSize: 18,
      sceneNameFontSize: 14,
      segmentFontSize: 12,
      resetButtonSize: 20,
      resetButtonPadding: 8,
      pauseButtonSize: 20,
      pauseButtonPadding: 8,
      bossOnlyButtonSize: 20,
      bossOnlyButtonPadding: 8,
      settingsButtonSize: 20,
      settingsButtonPadding: 8,
      minimizeButtonSize: 20,
      minimizeButtonPadding: 8,
      totalDamageLabelFontSize: 14,
      totalDamageValueFontSize: 18,
      totalDpsLabelFontSize: 14,
      totalDpsValueFontSize: 18,
      bossHealthLabelFontSize: 14,
      bossHealthNameFontSize: 14,
      bossHealthValueFontSize: 14,
      bossHealthPercentFontSize: 14,
      navTabFontSize: 11,
      navTabPaddingX: 14,
      navTabPaddingY: 6,
    },
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
    tableCustomization: new RuneStore(
      'liveTableCustomization',
      DEFAULT_SETTINGS.live.tableCustomization,
      RUNE_STORE_OPTIONS
    ),
    headerCustomization: new RuneStore(
      'liveHeaderCustomization',
      DEFAULT_SETTINGS.live.headerCustomization,
      RUNE_STORE_OPTIONS
    ),
    // Column order settings
    columnOrder: {
      dpsPlayers: new RuneStore('liveDpsPlayersColumnOrder', { order: DEFAULT_DPS_PLAYER_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
      dpsSkills: new RuneStore('liveDpsSkillsColumnOrder', { order: DEFAULT_DPS_SKILL_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
      healPlayers: new RuneStore('liveHealPlayersColumnOrder', { order: DEFAULT_HEAL_PLAYER_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
      healSkills: new RuneStore('liveHealSkillsColumnOrder', { order: DEFAULT_HEAL_SKILL_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
      tankedPlayers: new RuneStore('liveTankedPlayersColumnOrder', { order: DEFAULT_TANKED_PLAYER_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
      tankedSkills: new RuneStore('liveTankedSkillsColumnOrder', { order: DEFAULT_TANKED_SKILL_COLUMN_ORDER }, RUNE_STORE_OPTIONS),
    },
    // Sort settings
    sorting: {
      dpsPlayers: new RuneStore('liveDpsPlayersSorting', DEFAULT_LIVE_SORT_SETTINGS.dpsPlayers, RUNE_STORE_OPTIONS),
      dpsSkills: new RuneStore('liveDpsSkillsSorting', DEFAULT_LIVE_SORT_SETTINGS.dpsSkills, RUNE_STORE_OPTIONS),
      healPlayers: new RuneStore('liveHealPlayersSorting', DEFAULT_LIVE_SORT_SETTINGS.healPlayers, RUNE_STORE_OPTIONS),
      healSkills: new RuneStore('liveHealSkillsSorting', DEFAULT_LIVE_SORT_SETTINGS.healSkills, RUNE_STORE_OPTIONS),
      tankedPlayers: new RuneStore('liveTankedPlayersSorting', DEFAULT_LIVE_SORT_SETTINGS.tankedPlayers, RUNE_STORE_OPTIONS),
      tankedSkills: new RuneStore('liveTankedSkillsSorting', DEFAULT_LIVE_SORT_SETTINGS.tankedSkills, RUNE_STORE_OPTIONS),
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
  packetCapture: new RuneStore(
    'packetCapture',
    { method: "WinDivert", npcapDevice: "" },
    RUNE_STORE_OPTIONS
  ),
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
      tableCustomization: SETTINGS.live.tableCustomization.state,
      headerCustomization: SETTINGS.live.headerCustomization.state,
      columnOrder: {
        dpsPlayers: SETTINGS.live.columnOrder.dpsPlayers.state,
        dpsSkills: SETTINGS.live.columnOrder.dpsSkills.state,
        healPlayers: SETTINGS.live.columnOrder.healPlayers.state,
        healSkills: SETTINGS.live.columnOrder.healSkills.state,
        tankedPlayers: SETTINGS.live.columnOrder.tankedPlayers.state,
        tankedSkills: SETTINGS.live.columnOrder.tankedSkills.state,
      },
      sorting: {
        dpsPlayers: SETTINGS.live.sorting.dpsPlayers.state,
        dpsSkills: SETTINGS.live.sorting.dpsSkills.state,
        healPlayers: SETTINGS.live.sorting.healPlayers.state,
        healSkills: SETTINGS.live.sorting.healSkills.state,
        tankedPlayers: SETTINGS.live.sorting.tankedPlayers.state,
        tankedSkills: SETTINGS.live.sorting.tankedSkills.state,
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

// Theme selection removed — app uses only the `custom` theme controlled by customThemeColors
