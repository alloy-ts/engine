// src/core/config/seed.ts

/**
 * Requirement 3.17, 3.18, 3.19: Seed determinism
 */
export function randomizeOrder<T>(files: T[], seed: number): T[] {
	const result = [...files];
	let currentSeed = seed;

	// Simple LCG (Linear Congruential Generator) for determinism
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
