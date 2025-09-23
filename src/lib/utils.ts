export function formatElapsed(msBigInt: bigint) {
  const totalSeconds = Math.floor(Number(msBigInt) / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function pct(part: bigint, whole: bigint): number {
	if (!whole) return 0;
	return ((Number(part) / Number(whole)) * 100);
}

/** Abbreviates a number into more compact representation, returning an array with the truncated number and the abbreviation */
export function abbreviateNumberSplit(n: number): [number, string] {
  if (n >= 1e3 && n < 1e6) return [+(n / 1e3).toFixed(1), "k"];
  if (n >= 1e6 && n < 1e9) return [+(n / 1e6).toFixed(1), "m"];
  if (n >= 1e9 && n < 1e12) return [+(n / 1e9).toFixed(1), "b"];
  if (n >= 1e12) return [+(n / 1e12).toFixed(1), "t"];
  else return [+n.toFixed(0), ""];
}

export const classColors: Record<string, string> = {
  "Stormblade": "#674598",
  "Frost Mage": "#4de3d1",
  "Wind Knight": "#0099c6",
  "Verdant Oracle": "#66aa00",
  "Heavy Guardian": "#b38915",
  "Marksman": "#ffee00",
  "Shield Knight": "#7b9aa2",
  "Beat Performer": "#ee2e48",
};

export function getClassColor(className: string): string {
  return `rgb(from ${classColors[className] ?? "#ffc9ed"} r g b / 0.6)`;
}

export function getClassIcon(class_name: string): string {
  if (class_name == "") {
    return "/images/classes/blank.png";
  }
  return "/images/classes/" + class_name + ".png";
}