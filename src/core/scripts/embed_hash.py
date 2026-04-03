import sys, os, struct, hashlib

def blake3_compress(cv, block, counter, length, flags):
    IV = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
          0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19]
    MSG_SCHEDULE = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8],
        [3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1],
        [10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6],
        [12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4],
        [9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7],
        [11, 15, 5, 0, 1, 9, 8, 2, 14, 10, 2, 12, 3, 4, 7, 13]
    ]

    def rotr32(x, n): return ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF
    def g(state, a, b, c, d, m0, m1):
        state[a] = (state[a] + state[b] + m0) & 0xFFFFFFFF
        state[d] = rotr32(state[d] ^ state[a], 16)
        state[c] = (state[c] + state[d]) & 0xFFFFFFFF
        state[b] = rotr32(state[b] ^ state[c], 12)
        state[a] = (state[a] + state[b] + m1) & 0xFFFFFFFF
        state[d] = rotr32(state[d] ^ state[a], 8)
        state[c] = (state[c] + state[d]) & 0xFFFFFFFF
        state[b] = rotr32(state[b] ^ state[c], 7)

    m = list(struct.unpack('<16I', block))
    state = cv + IV[0:4] + [counter & 0xFFFFFFFF, (counter >> 32) & 0xFFFFFFFF, length, flags]

    for r in range(7):
        s = MSG_SCHEDULE[r]
        g(state, 0, 4, 8, 12, m[s[0]], m[s[1]])
        g(state, 1, 5, 9, 13, m[s[2]], m[s[3]])
        g(state, 2, 6, 10, 14, m[s[4]], m[s[5]])
        g(state, 3, 7, 11, 15, m[s[6]], m[s[7]])
        g(state, 0, 5, 10, 15, m[s[8]], m[s[9]])
        g(state, 1, 6, 11, 12, m[s[10]], m[s[11]])
        g(state, 2, 7, 8, 13, m[s[12]], m[s[13]])
        g(state, 3, 4, 9, 14, m[s[14]], m[s[15]])

    return [(state[i] ^ state[i+8]) for i in range(8)]

def blake3_hash(data):
    cv = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
          0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19]
    block = data[:64].ljust(64, b'\0')
    cv = blake3_compress(cv, block, 0, min(len(data), 64), 0)
    return struct.pack('<8I', *cv)

def embed_elf(binary_path, symbol_name):
    from elftools.elf.elffile import ELFFile
    from elftools.elf.sections import SymbolTableSection

    with open(binary_path, 'rb') as f:
        elf = ELFFile(f)
        symtab = elf.get_section_by_name('.symtab')
        if not symtab:
            print("No symbol table found")
            return

        symbol = None
        for sym in symtab.iter_symbols():
            if sym.name == symbol_name:
                symbol = sym
                break

        if not symbol:
            print(f"Symbol {symbol_name} not found")
            return

        section_index = symbol['st_shndx']
        section = elf.get_section(section_index)
        file_offset = section['sh_offset'] + (symbol['st_value'] - section['sh_addr'])

    with open(binary_path, 'r+b') as f:
        f.seek(file_offset)
        f.write(b'\x00' * 32)
        f.seek(0)
        data = f.read()
        h = blake3_hash(data)
        f.seek(file_offset)
        f.write(h)
    print(f"Embedded hash at offset {file_offset}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: embed_hash.py <binary> <symbol>")
        sys.exit(1)

    binary_path = sys.argv[1]
    symbol_name = sys.argv[2]

    if sys.platform == 'linux' or sys.platform.startswith('linux'):
        embed_elf(binary_path, symbol_name)
    else:
        print(f"Platform {sys.platform} not yet supported for hash embedding")
        sys.exit(1)
