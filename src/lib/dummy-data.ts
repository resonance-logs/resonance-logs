import type { PlayerRow, SkillRow, PlayersWindow, HeaderInfo } from "$lib/api";

export const DUMMY_PLAYER_DATA: PlayerRow[] = [
  {
    uid: 312141,
    name: "Stormblade",
    className: "Stormblade",
    classSpecName: "Iaido",
    abilityScore: 50000,
    totalDmg: 100000,
    dps: 8333,
    dmgPct: 22.5,
    critRate: 45.2,
    critDmgRate: 68.5,
    luckyRate: 15.3,
    luckyDmgRate: 12.8,
    hits: 450,
    hitsPerMinute: 225,
  },
  {
    uid: 312142,
    name: "Frost Mage",
    className: "Frost Mage",
    classSpecName: "Icicle",
    abilityScore: 45000,
    totalDmg: 90000,
    dps: 7500,
    dmgPct: 20.2,
    critRate: 42.8,
    critDmgRate: 65.3,
    luckyRate: 14.1,
    luckyDmgRate: 11.5,
    hits: 380,
    hitsPerMinute: 190,
  },
  {
    uid: 312143,
    name: "Wind Knight",
    className: "Wind Knight",
    classSpecName: "Vanguard",
    abilityScore: 40000,
    totalDmg: 80000,
    dps: 6667,
    dmgPct: 18.0,
    critRate: 38.5,
    critDmgRate: 61.2,
    luckyRate: 13.2,
    luckyDmgRate: 10.8,
    hits: 420,
    hitsPerMinute: 210,
  },
  {
    uid: 312144,
    name: "Verdant Oracle",
    className: "Verdant Oracle",
    classSpecName: "Smite",
    abilityScore: 35000,
    totalDmg: 70000,
    dps: 5833,
    dmgPct: 15.7,
    critRate: 35.8,
    critDmgRate: 58.9,
    luckyRate: 12.5,
    luckyDmgRate: 9.8,
    hits: 340,
    hitsPerMinute: 170,
  },
  {
    uid: 312145,
    name: "Heavy Guardian",
    className: "Heavy Guardian",
    classSpecName: "Earthfort",
    abilityScore: 30000,
    totalDmg: 60000,
    dps: 5000,
    dmgPct: 13.5,
    critRate: 32.4,
    critDmgRate: 55.6,
    luckyRate: 11.3,
    luckyDmgRate: 8.9,
    hits: 310,
    hitsPerMinute: 155,
  },
  {
    uid: 312146,
    name: "Marksman",
    className: "Marksman",
    classSpecName: "Wildpack",
    abilityScore: 25000,
    totalDmg: 50000,
    dps: 4167,
    dmgPct: 11.2,
    critRate: 48.9,
    critDmgRate: 72.3,
    luckyRate: 16.8,
    luckyDmgRate: 13.5,
    hits: 520,
    hitsPerMinute: 260,
  },
  {
    uid: 312147,
    name: "Shield Knight",
    className: "Shield Knight",
    classSpecName: "Recovery",
    abilityScore: 20000,
    totalDmg: 40000,
    dps: 3333,
    dmgPct: 9.0,
    critRate: 28.7,
    critDmgRate: 52.1,
    luckyRate: 9.8,
    luckyDmgRate: 7.5,
    hits: 280,
    hitsPerMinute: 140,
  },
  {
    uid: 312148,
    name: "Beat Performer",
    className: "Beat Performer",
    classSpecName: "Dissonance",
    abilityScore: 15000,
    totalDmg: 30000,
    dps: 2500,
    dmgPct: 6.7,
    critRate: 25.3,
    critDmgRate: 48.7,
    luckyRate: 8.5,
    luckyDmgRate: 6.2,
    hits: 240,
    hitsPerMinute: 120,
  },
];

// Generate skills for a given player
export function generateDummySkills(playerUid: number): SkillRow[] {
  const player = DUMMY_PLAYER_DATA.find(p => p.uid === playerUid);
  if (!player) return [];

  const skillCount = 8;
  const skills: SkillRow[] = [];

  for (let i = 1; i <= skillCount; i++) {
    const dmgPct = Math.random() * 20 + (30 - i * 3);
    const totalDmg = (player.totalDmg * dmgPct) / 100;

    skills.push({
      name: `Skill ${i}`,
      totalDmg: Math.floor(totalDmg),
      dps: Math.floor(totalDmg / 12), // Assuming 12 second duration
      dmgPct: Math.round(dmgPct * 10) / 10,
      critRate: Math.random() * 30 + 30,
      critDmgRate: Math.random() * 30 + 50,
      luckyRate: Math.random() * 15 + 5,
      luckyDmgRate: Math.random() * 10 + 5,
      hits: Math.floor(Math.random() * 50) + 20,
      hitsPerMinute: Math.floor(Math.random() * 30) + 10,
    });
  }

  // Sort by totalDmg descending
  skills.sort((a, b) => b.totalDmg - a.totalDmg);

  return skills;
}

export function generateDummyPlayersWindow(): PlayersWindow {
  return {
    playerRows: DUMMY_PLAYER_DATA,
  };
}

// Dummy header data for preview mode
export const DUMMY_HEADER_INFO: HeaderInfo = {
  totalDps: 45833,
  totalDmg: 2750000,
  elapsedMs: 4020000, // 67:00
  fightStartTimestampMs: 0,
  bosses: [
    {
      uid: 999999,
      name: "Test Boss",
      currentHp: 1500000,
      maxHp: 3000000,
    },
  ],
  sceneId: 1001,
  sceneName: "Test Dungeon",
  currentSegmentType: "boss",
  currentSegmentName: "Test Boss",
};

// Dummy segment info for preview mode
export const DUMMY_SEGMENT_INFO = {
  durationSecs: 67,
  type: "boss" as const,
  label: "Test Boss",
};