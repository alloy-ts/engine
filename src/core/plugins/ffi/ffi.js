// ffi.js — alloy:ffi JS entry point

export const FFIType = {
    void: 'void', bool: 'bool',
    u8: 'u8', u16: 'u16', u32: 'u32', u64: 'u64',
    i8: 'i8', i16: 'i16', i32: 'i32', i64: 'i64',
    f32: 'f32', f64: 'f64',
    ptr: 'ptr', cstring: 'cstring',
    function: 'function',
};

export function dlopen(path, symbols) {
    const handleId = JSON.parse(
        __alloy_ffi_dlopen(path, JSON.stringify(symbols))
    );
    if (handleId && typeof handleId === 'object' && handleId.error) throw new Error(handleId.error);

    const lib = {};
    for (const [name, def] of Object.entries(symbols)) {
        lib[name] = (...args) => {
            const result = JSON.parse(
                __alloy_ffi_call(handleId, name, JSON.stringify(args), def.returns)
            );
            if (result && typeof result === 'object' && result.error) throw new Error(result.error);
            return result;
        };
    }
    return lib;
}

export const AlloyFFI = { FFIType, dlopen };
export default AlloyFFI;
