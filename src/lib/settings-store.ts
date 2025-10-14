import { version } from '@tauri-apps/plugin-os';
import { RuneStore } from '@tauri-store/svelte';

const IS_WIN_11 = parseInt(version().split(".")[2] || "0", 10) >= 22000;

const DEFAULT_SETTINGS = {
  general: {
    showYourName: "Show Your Name",
    showOthersName: "Show Others' Name",
    showYourAbilityScore: true,
    showOthersAbilityScore: true,
    relativeToTop: false,
  },
  accessibility: {
    blur: !IS_WIN_11,
    transparency: false,
  },
  shortcuts: {
    showMeter: "",
    hideMeter: "",
    enableClickthrough: "",
    disableClickthrough: "",
  },
  live: {
    dps: {
      players: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: false,
        hitsPerMinute: false,
        luckyDmgRate: false,
        luckyRate: false,
        totalDmg: true,
      },
      skillBreakdown: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: false,
        hitsPerMinute: false,
        luckyDmgRate: false,
        luckyRate: false,
        totalDmg: true,
      },
    },
    heal: {
      players: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: false,
        hitsPerMinute: false,
        luckyDmgRate: false,
        luckyRate: false,
        totalDmg: true,
      },
      skillBreakdown: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: false,
        hitsPerMinute: false,
        luckyDmgRate: false,
        luckyRate: false,
        totalDmg: true,
      },
    },
  },
};

export const settings = new RuneStore('settings', DEFAULT_SETTINGS, {
  autoStart: true,
  saveOnChange: true,
});
