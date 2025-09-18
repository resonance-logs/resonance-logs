export function pct(part: bigint, whole: bigint) {
	if (!whole) return "0.00%";
	return `${((Number(part) / Number(whole)) * 100).toFixed(2)}%`;
}
