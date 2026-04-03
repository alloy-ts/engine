import sys, os, struct, hashlib

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

        offset = symbol['st_value'] # This is VMA, we need file offset
        # For simplicity in static PIE, sometimes VMA matches file offset relative to segment
        # but better to find the section and calculate.
        section_index = symbol['st_shndx']
        section = elf.get_section(section_index)
        file_offset = section['sh_offset'] + (symbol['st_value'] - section['sh_addr'])

    with open(binary_path, 'r+b') as f:
        # Zero the hash field
        f.seek(file_offset)
        f.write(b'\x00' * 32)

        # Compute hash
        f.seek(0)
        data = f.read()
        # Mock BLAKE3 matching C implementation (with Rotation)
        cv = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19]
        def rotr32(x, n): return ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF
        for i, b in enumerate(data):
            cv[i % 8] ^= b
            cv[i % 8] = rotr32(cv[i % 8], 7)
        h = struct.pack('<8I', *cv)

        # Write back
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
