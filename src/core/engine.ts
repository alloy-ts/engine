import { createHash } from "node:crypto";
import {
	mkdir,
	chmod,
	unlink,
	access,
	readFile,
	writeFile,
	existsSync,
	mkdirSync,
	readFileSync,
} from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

export abstract class Loader {
	abstract load(url: string): Promise<string>;
}

export abstract class Transpiler {
	abstract transformSync(code: string, loader?: Loader): string;
}

export function getPlatformInfo() {
	const platform = process.platform;
	const arch = process.arch;
	return { platform, arch };
}

export class BinaryManager {
	private cacheDir: string;

	constructor() {
		const { platform, arch } = getPlatformInfo();
		this.cacheDir = path.join(
			os.homedir(),
			".alloy",
			"bin",
			`${platform}-${arch}`,
		);
	}

	async ensureBinary(version: string): Promise<string> {
		const { platform, arch } = getPlatformInfo();
		const ext = platform === "win32" ? ".exe" : "";
		const binaryName = `alloy_host-${platform}-${arch}${ext}`;
		const binaryPath = path.join(this.cacheDir, "alloy_host" + ext);

		if (await this.isCached(binaryPath)) {
			if (await this.verifyChecksum(binaryPath)) {
				return binaryPath;
			}
		}

		await mkdir(this.cacheDir, { recursive: true });
		await this.downloadBinary(version, binaryName, binaryPath);

		if (!(await this.verifyChecksum(binaryPath))) {
			await unlink(binaryPath);
			throw new Error("Checksum verification failed for alloy_host binary");
		}

		await chmod(binaryPath, 0o755);
		return binaryPath;
	}

	private async isCached(filePath: string): Promise<boolean> {
		try {
			await access(filePath);
			return true;
		} catch {
			return false;
		}
	}

	private async downloadBinary(
		version: string,
		name: string,
		dest: string,
	): Promise<void> {
		const url = `https://github.com/alloyscript/engine/releases/download/v${version}/${name}`;
		console.log(`Downloading alloy_host from ${url}...`);

		const response = await fetch(url);
		if (!response.ok)
			throw new Error(`Failed to download binary: ${response.statusText}`);
		const arrayBuffer = await response.arrayBuffer();
		await writeFile(dest, Buffer.from(arrayBuffer));

		const shaResponse = await fetch(`${url}.sha256`);
		if (!shaResponse.ok) throw new Error("Checksum file not found");
		const shaText = await shaResponse.text();
		await writeFile(dest + ".sha256", shaText.trim());
	}

	private async verifyChecksum(binaryPath: string): Promise<boolean> {
		try {
			const content = await readFile(binaryPath);
			const hash = createHash("sha256").update(content).digest("hex");
			const expectedFileContent = await readFile(
				binaryPath + ".sha256",
				"utf-8",
			);
			const expectedHash = expectedFileContent.split(/\s+/)[0];
			return hash === expectedHash;
		} catch {
			return false;
		}
	}
}

export class Blob {
	#parts: (string | ArrayBuffer | ArrayBufferView | Blob)[];
	#type: string;
	#size: number;

	constructor(
		parts: (string | ArrayBuffer | ArrayBufferView | Blob)[],
		options: { type?: string } = {},
	) {
		this.#parts = parts;
		this.#type = options.type || "";
		this.#size = parts.reduce((acc, part) => {
			if (typeof part === "string") {
				// Calculate UTF-8 byte length
				return acc + new TextEncoder().encode(part).length;
			}
			if (part instanceof Blob) return acc + part.size;
			return acc + (part as ArrayBufferView).byteLength;
		}, 0);
	}

	get size(): number {
		return this.#size;
	}

	get type(): string {
		return this.#type;
	}

	async arrayBuffer(): Promise<ArrayBuffer> {
		const sink = new ArrayBufferSink();
		sink.start({});
		for (const part of this.#parts) {
			if (typeof part === "string") {
				sink.write(part);
			} else if (part instanceof Blob) {
				sink.write(await part.arrayBuffer());
			} else {
				sink.write(part as any);
			}
		}
		return sink.end() as ArrayBuffer;
	}

	async text(): Promise<string> {
		const buf = await this.arrayBuffer();
		return new TextDecoder().decode(buf);
	}
}

export class File extends Blob {
	#name: string;
	#lastModified: number;

	constructor(
		parts: any[],
		name: string,
		options: { type?: string; lastModified?: number } = {},
	) {
		super(parts, options);
		this.#name = name;
		this.#lastModified = options.lastModified || Date.now();
	}

	get name() {
		return this.#name;
	}
	get lastModified() {
		return this.#lastModified;
	}
}

/**
 * Native lazy-loaded file wrapper.
 * Created with Alloy.file(path).
 */
export class AlloyFile extends Blob {
	#path: string;

	constructor(path: string) {
		super([]);
		this.#path = path;
	}

	get path() {
		return this.#path;
	}

	async text() {
		return Alloy.readFile(this.#path);
	}

	get size(): number {
		return Alloy.getFileSize(this.#path);
	}
}

/**
 * Fast incremental writer for constructing ArrayBuffers.
 * Proxies to the high-performance C implementation.
 */
export class ArrayBufferSink {
	#native: any;

	constructor() {
		this.#native = Alloy.createArrayBufferSink();
	}

	start(options?: any) {
		Alloy.sinkStart(this.#native, options);
	}

	write(chunk: string | ArrayBufferView | ArrayBuffer): number {
		return Alloy.sinkWrite(this.#native, chunk);
	}

	flush(): number | Uint8Array | ArrayBuffer {
		return Alloy.sinkFlush(this.#native);
	}

	end(): number | Uint8Array | ArrayBuffer {
		return Alloy.sinkEnd(this.#native);
	}
}

(Alloy as any).file = (path: string) => new AlloyFile(path);
