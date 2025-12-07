#!/usr/bin/env node
// CommonJS variant for projects using "type": "module"
const fs = require('fs');
const path = require('path');

const inputPath = process.argv[2] || path.join('src-tauri', 'meter-data', 'BuffName.json');
const outputPath = process.argv[3] || path.join('src-tauri', 'meter-data', 'BuffName.simplified.json');
const overwrite = process.argv.includes('--overwrite');

function isString(x){ return typeof x === 'string' && x.trim() !== ''; }

function collectNames(obj, acc = {englishShorts: [], englishLongs: [], aiShorts: [], aiLongs: [], chineseShorts: [], manualShort: null, manualLong: null}){
  if (!obj || typeof obj !== 'object') return acc;
  for (const [k,v] of Object.entries(obj)){
    const key = String(k).toLowerCase();
    if (isString(v)){
      if (key === 'englishshortmanualoverride' || key === 'englishshortmanual' || key === 'englishshort_manualoverride') acc.manualShort = v;
      else if (key === 'englishlongmanualoverride' || key === 'englishlongmanual' || key === 'englishlong_manualoverride') acc.manualLong = v;
      else if (key === 'englishshort') acc.englishShorts.push(v);
      else if (key === 'englishlong') acc.englishLongs.push(v);
      else if (key === 'aienglishshort') acc.aiShorts.push(v);
      else if (key === 'aienglishlong') acc.aiLongs.push(v);
      else if (key === 'chineseshort') acc.chineseShorts.push(v);
    } else if (typeof v === 'object'){
      collectNames(v, acc);
    }
  }
  return acc;
}

function pickName(id, entry){
  const c = collectNames(entry || {});
  const englishShort = c.manualShort || c.englishShorts.find(isString) || null;
  const englishLong = c.manualLong || c.englishLongs.find(isString) || null;

  const finalShort = englishShort || `Unknown Skill - #${id}`;
  const finalLong = englishLong || finalShort;

  const hasEnglishShortAndLong = Boolean((englishShort) && (englishLong));

  return {
    ChineseShort: c.chineseShorts.find(isString) || null,
    AIEnglishShort: c.aiShorts.find(isString) || null,
    AIEnglishLong: c.aiLongs.find(isString) || null,
    EnglishShort: finalShort,
    EnglishLong: finalLong,
    hasEnglishShortAndLong: hasEnglishShortAndLong
  };
}

try{
  const raw = fs.readFileSync(inputPath, 'utf8');
  const data = JSON.parse(raw);
  const out = {};
  for (const id of Object.keys(data)){
    out[id] = pickName(id, data[id]);
  }

  if (fs.existsSync(outputPath) && !overwrite){
    console.error('Output exists. Use --overwrite to replace it or pass a different output path.');
    process.exitCode = 2;
    process.exit();
  }

  fs.writeFileSync(outputPath, JSON.stringify(out, null, 2), 'utf8');
  console.log('Wrote simplified file to', outputPath);
} catch (err){
  console.error('Error:', err && err.message ? err.message : err);
  process.exit(1);
}
