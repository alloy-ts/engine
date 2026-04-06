// src/core/config/seed.ts

/**
 * Linear Congruential Generator (LCG) for deterministic randomization
 */
export class DeterministicRandom {
	private state: number;

	constructor(seed: number) {
		this.state = seed % 2147483647;
		if (this.state <= 0) this.state += 2147483646;
	}

	next(): number {
		this.state = (this.state * 16807) % 2147483647;
		return (this.state - 1) / 2147483646;
	}
}

/**
 * Randomize order of files using a seed (Requirement 3.19)
 */
export function randomizeOrder<T>(items: T[], seed: number): T[] {
	const rng = new DeterministicRandom(seed);
	const result = [...items];

	for (let i = result.length - 1; i > 0; i--) {
		const j = Math.floor(rng.next() * (i + 1));
		[result[i], result[j]] = [result[j], result[i]];
	}

	return result;
}
