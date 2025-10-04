import { RuneStore } from '@tauri-store/svelte';

const DEFAULT_SETTINGS = {
  general: {

  },
  accessibility: {

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
