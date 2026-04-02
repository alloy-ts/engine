export enum FFIType {
	buffer = 0,
	cstring = 1,
	function = 2,
	ptr = 3,
	i8 = 4,
	i16 = 5,
	i32 = 6,
	i64 = 7,
	i64_fast = 8,
	u8 = 9,
	u16 = 10,
	u32 = 11,
	u64 = 12,
	u64_fast = 13,
	f32 = 14,
	f64 = 15,
	bool = 16,
	char = 17,
	napi_env = 18,
	napi_value = 19,
}

export class CString {
	constructor(
		public ptr: number,
		public byteOffset?: number,
		public byteLength?: number,
	) {}
	toString(): string {
		return (globalThis as any).AlloyInternal.ffi.readCString(
			this.ptr,
			this.byteOffset,
			this.byteLength,
		);
	}
}

export class JSCallback {
	readonly ptr: number;
	constructor(
		fn: Function,
		options: { returns: FFIType; args: FFIType[]; threadsafe?: boolean },
	) {
		this.ptr = (globalThis as any).AlloyInternal.ffi.createCallback(
			fn,
			options,
		);
	}
	close(): void {
		(globalThis as any).AlloyInternal.ffi.closeCallback(this.ptr);
	}
}

export class CFunction {
	constructor(options: { returns: FFIType; args: FFIType[]; ptr: number }) {
		const wrapper = (...args: any[]) =>
			(globalThis as any).AlloyInternal.ffi.callNative(
				options.ptr,
				options.args,
				options.returns,
				args,
			);
		return wrapper as any;
	}
}

export function dlopen(
	path: string,
	symbols: Record<string, { returns: FFIType; args: FFIType[] }>,
) {
	return (globalThis as any).AlloyInternal.ffi.dlopen(path, symbols);
}

export function ptr(typedArray: any): number {
	return (globalThis as any).AlloyInternal.ffi.getPtr(typedArray);
}

export const suffix = (globalThis as any).AlloyInternal.ffi.suffix;

export function dlopen(path: string, symbols: any) {
	return (globalThis as any).AlloyInternal.dlopen(path, symbols);
}

export function linkSymbols(symbols: any) {
	return (globalThis as any).AlloyInternal.linkSymbols(symbols);
}

export const suffix =
	process.platform === "win32"
		? "dll"
		: process.platform === "darwin"
			? "dylib"
			: "so";

export function ptr(typedArray: any): number {
	return (globalThis as any).AlloyInternal.ptr(typedArray);
}

export function toArrayBuffer(
	ptr: number,
	byteOffset?: number,
	byteLength?: number,
	deallocatorContext?: any,
	deallocator?: any,
): ArrayBuffer {
	return (globalThis as any).AlloyInternal.toArrayBuffer(
		ptr,
		byteOffset,
		byteLength,
		deallocatorContext,
		deallocator,
	);
}

export const read = {
	u8: (ptr: number, offset: number) =>
		(globalThis as any).AlloyInternal.readU8(ptr, offset),
	i32: (ptr: number, offset: number) =>
		(globalThis as any).AlloyInternal.readI32(ptr, offset),
	f64: (ptr: number, offset: number) =>
		(globalThis as any).AlloyInternal.readF64(ptr, offset),
};

export class CString {
	ptr: number;
	constructor(ptr: number) {
		this.ptr = ptr;
	}
}

export class JSCallback {
	ptr: number;
	constructor(fn: Function, options: any) {
		this.ptr = 0; // Mocked
	}
	close() {}
}
