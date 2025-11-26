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
  playerTextColor: "#ffffff",
  
  // Table header settings
  showTableHeader: true,
  tableHeaderHeight: 24,
  tableHeaderFontSize: 11,
  tableHeaderTextColor: "#a1a1aa",
  
  // Abbreviated numbers (K, M, %)
  abbreviatedFontSize: 10,
  abbreviatedColor: "#71717a",
  
  // Skill row settings (separate from player rows)
  skillRowHeight: 24,
  skillFontSize: 12,
  skillIconSize: 18,
  skillTextColor: "#ffffff",
  skillShowHeader: true,
  skillHeaderHeight: 22,
  skillHeaderFontSize: 10,
  skillHeaderTextColor: "#a1a1aa",
  skillAbbreviatedFontSize: 9,
  skillAbbreviatedColor: "#71717a",
};

// Header customization defaults
export const DEFAULT_HEADER_SETTINGS = {
  // Layout settings
  showHeader: true,
  windowPadding: 12, // padding around entire live meter window
  headerPadding: 8, // internal padding within header
  
  // Row 1 elements (left side)
  showTimer: true,
  showSceneName: true,
  showSegmentInfo: true,
  
  // Row 1 elements (right side - control buttons)
  showResetButton: true,
  showPauseButton: true,
  showBossOnlyButton: true,
  showSettingsButton: true,
  showMinimizeButton: true,
  
  // Row 2 elements (left side - stats)
  showTotalDamage: true,
  showTotalDps: true,
  showBossHealth: true,
  
  // Row 2 elements (right side - navigation tabs)
  showNavigationTabs: true,
  
  // Timer sizing
  timerLabelFontSize: 12,
  timerFontSize: 18,
  
  // Scene name sizing
  sceneNameFontSize: 14,
  
  // Segment info sizing
  segmentFontSize: 12,
  
  // Control button sizing (individual)
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
  
  // Stats sizing
  totalDamageLabelFontSize: 14,
  totalDamageValueFontSize: 18,
  totalDpsLabelFontSize: 14,
  totalDpsValueFontSize: 18,
  bossHealthLabelFontSize: 14,
  
  // Boss health sizing (granular)
  bossHealthNameFontSize: 14,
  bossHealthValueFontSize: 14,
  bossHealthPercentFontSize: 14,
  
  // Navigation tabs sizing
  navTabFontSize: 11,
  navTabPaddingX: 14,
  navTabPaddingY: 6,
};

// Header presets
export const HEADER_PRESETS = {
  full: {
    showHeader: true,
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
  compact: {
    showHeader: true,
    windowPadding: 8,
    headerPadding: 4,
    showTimer: true,
    showSceneName: true,
    showSegmentInfo: false,
    showResetButton: true,
    showPauseButton: true,
    showBossOnlyButton: true,
    showSettingsButton: false,
    showMinimizeButton: false,
    showTotalDamage: false,
    showTotalDps: false,
    showBossHealth: false,
    showNavigationTabs: false,
    timerLabelFontSize: 0,
    timerFontSize: 16,
    sceneNameFontSize: 14,
    segmentFontSize: 11,
    resetButtonSize: 18,
    resetButtonPadding: 6,
    pauseButtonSize: 18,
    pauseButtonPadding: 6,
    bossOnlyButtonSize: 18,
    bossOnlyButtonPadding: 6,
    settingsButtonSize: 18,
    settingsButtonPadding: 6,
    minimizeButtonSize: 18,
    minimizeButtonPadding: 6,
    totalDamageLabelFontSize: 12,
    totalDamageValueFontSize: 16,
    totalDpsLabelFontSize: 12,
    totalDpsValueFontSize: 16,
    bossHealthLabelFontSize: 12,
    bossHealthNameFontSize: 12,
    bossHealthValueFontSize: 12,
    bossHealthPercentFontSize: 12,
    navTabFontSize: 10,
    navTabPaddingX: 10,
    navTabPaddingY: 4,
  },
  none: {
    showHeader: false,
    windowPadding: 0,
    headerPadding: 0,
    showTimer: false,
    showSceneName: false,
    showSegmentInfo: false,
    showResetButton: false,
    showPauseButton: false,
    showBossOnlyButton: false,
    showSettingsButton: false,
    showMinimizeButton: false,
    showTotalDamage: false,
    showTotalDps: false,
    showBossHealth: false,
    showNavigationTabs: false,
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
};

export const FONT_SIZE_LABELS: Record<string, string> = {
  xs: 'Extra Small',
  sm: 'Small',
  base: 'Base',
  lg: 'Large',
  xl: 'Extra Large',
};

// Default custom theme colors (based on dark theme)
export const DEFAULT_CUSTOM_THEME_COLORS: Record<string, string> = {
  background: 'rgba(33, 33, 33, 1)',
  foreground: 'rgba(226, 226, 226, 1)',
  card: 'rgba(41, 41, 41, 1)',
  cardForeground: 'rgba(226, 226, 226, 1)',
  popover: 'rgba(41, 41, 41, 1)',
  popoverForeground: 'rgba(226, 226, 226, 1)',
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
  ring: 'rgba(166, 166, 166, 1)',
  chart1: 'rgba(166, 166, 166, 1)',
  chart2: 'rgba(120, 180, 200, 1)',
  chart3: 'rgba(120, 120, 120, 1)',
  chart4: 'rgba(100, 100, 100, 1)',
  chart5: 'rgba(82, 82, 82, 1)',
  sidebar: 'rgba(38, 38, 38, 1)',
  sidebarForeground: 'rgba(226, 226, 226, 1)',
  sidebarPrimary: 'rgba(166, 166, 166, 1)',
  sidebarPrimaryForeground: 'rgba(33, 33, 33, 1)',
  sidebarAccent: 'rgba(82, 82, 82, 1)',
  sidebarAccentForeground: 'rgba(226, 226, 226, 1)',
  sidebarBorder: 'rgba(74, 74, 74, 1)',
  sidebarRing: 'rgba(166, 166, 166, 1)',
};

// Labels for custom theme color variables
export const CUSTOM_THEME_COLOR_LABELS: Record<string, { label: string; description: string; category: string }> = {
  background: { label: 'Background', description: 'Main app background color', category: 'Base' },
  foreground: { label: 'Foreground', description: 'Main text color', category: 'Base' },
  card: { label: 'Card', description: 'Card/panel background', category: 'Surfaces' },
  cardForeground: { label: 'Card Text', description: 'Text on cards', category: 'Surfaces' },
  popover: { label: 'Popover', description: 'Popup/dropdown background', category: 'Surfaces' },
  popoverForeground: { label: 'Popover Text', description: 'Text in popups', category: 'Surfaces' },
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
  ring: { label: 'Ring', description: 'Focus ring color', category: 'Utility' },
  chart1: { label: 'Chart 1', description: 'Chart color 1', category: 'Charts' },
  chart2: { label: 'Chart 2', description: 'Chart color 2', category: 'Charts' },
  chart3: { label: 'Chart 3', description: 'Chart color 3', category: 'Charts' },
  chart4: { label: 'Chart 4', description: 'Chart color 4', category: 'Charts' },
  chart5: { label: 'Chart 5', description: 'Chart color 5', category: 'Charts' },
  sidebar: { label: 'Sidebar', description: 'Sidebar background', category: 'Sidebar' },
  sidebarForeground: { label: 'Sidebar Text', description: 'Sidebar text color', category: 'Sidebar' },
  sidebarPrimary: { label: 'Sidebar Primary', description: 'Sidebar primary accent', category: 'Sidebar' },
  sidebarPrimaryForeground: { label: 'Sidebar Primary Text', description: 'Text on sidebar primary', category: 'Sidebar' },
  sidebarAccent: { label: 'Sidebar Accent', description: 'Sidebar accent color', category: 'Sidebar' },
  sidebarAccentForeground: { label: 'Sidebar Accent Text', description: 'Text on sidebar accent', category: 'Sidebar' },
  sidebarBorder: { label: 'Sidebar Border', description: 'Sidebar border color', category: 'Sidebar' },
  sidebarRing: { label: 'Sidebar Ring', description: 'Sidebar focus ring', category: 'Sidebar' },
};

const DEFAULT_SETTINGS = {
  accessibility: {
    theme: 'dark' as string,
    blur: false,
    transparency: false,
    transparentOpacityPercent: 2,
    transparentMode: false,
    clickthrough: false,
    classColors: { ...DEFAULT_CLASS_COLORS },
    useClassSpecColors: false,
    classSpecColors: { ...DEFAULT_CLASS_SPEC_COLORS },
    fontSizes: { ...DEFAULT_FONT_SIZES },
    customThemeColors: { ...DEFAULT_CUSTOM_THEME_COLORS },
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
    tableCustomization: { ...DEFAULT_LIVE_TABLE_SETTINGS },
    headerCustomization: { ...DEFAULT_HEADER_SETTINGS },
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
      tableCustomization: SETTINGS.live.tableCustomization.state,
      headerCustomization: SETTINGS.live.headerCustomization.state,
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
  'rainbow',
  'custom'
];
