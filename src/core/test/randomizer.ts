// src/core/test/randomizer.ts

export function randomizeOrder<T>(items: T[], seed: number): T[] {
	const result = [...items];
	let currentSeed = seed;

	const nextRandom = () => {
		currentSeed = (currentSeed * 9301 + 49297) % 233280;
		return currentSeed / 233280;
	};

	// Fisher-Yates shuffle
	for (let i = result.length - 1; i > 0; i--) {
		const j = Math.floor(nextRandom() * (i + 1));
		[result[i], result[j]] = [result[j], result[i]];
	}

	return result;
}
