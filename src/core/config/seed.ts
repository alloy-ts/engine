/**
 * Deterministically randomizes the order of an array using a seed.
 * Uses a Linear Congruential Generator (LCG).
 */
export function randomizeOrder<T>(array: T[], seed: number): T[] {
	const result = [...array];
	let currentSeed = seed;

	const nextRandom = () => {
		currentSeed = (currentSeed * 1664525 + 1013904223) % 4294967296;
		return currentSeed / 4294967296;
	};

	for (let i = result.length - 1; i > 0; i--) {
		const j = Math.floor(nextRandom() * (i + 1));
		[result[i], result[j]] = [result[j], result[i]];
	}

	return result;
}
