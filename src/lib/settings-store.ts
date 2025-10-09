import { version } from '@tauri-apps/plugin-os';
import { RuneStore } from '@tauri-store/svelte';

const IS_WIN_11 = parseInt(version().split(".")[2] || "0", 10) >= 22000;

const DEFAULT_SETTINGS = {
  general: {
    showYourName: "Show Your Name",
    showOthersName: "Show Others' Name",
    showYourAbilityScore: true,
    showOthersAbilityScore: true,
  },
  accessibility: {
    blur: !IS_WIN_11,
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
        hits: true,
        hitsPerMinute: true,
        luckyDmgRate: true,
        luckyRate: true,
        totalDmg: true,
      },
      skillBreakdown: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: true,
        hitsPerMinute: true,
        luckyDmgRate: true,
        luckyRate: true,
        totalDmg: true,
      },
    },
    heal: {
      players: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: true,
        hitsPerMinute: true,
        luckyDmgRate: true,
        luckyRate: true,
        totalDmg: true,
      },
      skillBreakdown: {
        critDmgRate: true,
        critRate: true,
        dmgPct: true,
        dps: true,
        hits: true,
        hitsPerMinute: true,
        luckyDmgRate: true,
        luckyRate: true,
        totalDmg: true,
      },
    },
  },
};

export const settings = new RuneStore('settings', DEFAULT_SETTINGS, {
  autoStart: true,
  saveOnChange: true,
});
