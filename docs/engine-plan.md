# Engine plan

## Implemented (MVP)

Language subset, following ECMA-262 semantics for these primitives:

- Values: undefined, null, boolean, number (f64), string.
- Literals: decimal numbers with fraction and exponent, `0x`/`0o`/`0b`
  integers; `'...'` and `"..."` strings with `\n \t \r \b \f \v \0 \' \" \\`,
  `\xHH`, `\uHHHH`, `\u{...}` (surrogate pairs combined), line continuations,
  and identity escapes; `true`, `false`, `null`.
- Identifiers resolved from Realm globals; an unknown name is a
  ReferenceError (`x is not defined`). Seeded read-only globals: `undefined`,
  `NaN`, `Infinity`.
- Parentheses; unary `+ - !`; binary `* / %`, `+ -`, `< <= > >=`,
  `== != === !==` with ECMA-262 precedence and left associativity.
- Scripts of expression statements separated by `;` or by a newline where
  automatic semicolon insertion applies (`1\n+2` is one statement, as in
  ECMAScript). Empty statements are allowed. The completion value is the last
  evaluated statement's value, or undefined.
- `//` and `/* */` comments.
- `+` concatenates when either operand is a string; relational comparison of
  two strings uses UTF-16 code units; IsLooselyEqual covers null/undefined
  and number/string/boolean coercion; NaN, ±Infinity and -0 follow IEEE 754
  and the spec (`+0 === -0`, `NaN !== NaN`, `-4 % 2` is `-0`).
- Number-to-string uses `Number::toString` formatting (`1e+21`, `1e-7`,
  `"0"` for -0); string-to-number uses StringToNumber (whitespace trimming,
  `Infinity`, hex/octal/binary prefixes, `""` is 0).
- Malformed source is a SyntaxError before any statement runs; the parser
  never panics.

Rejected with SyntaxError because they are outside the subset: reserved words
(`typeof`, `if`, `var`, ...), assignment, `++`/`--`, `**`, shifts, logical
operators, calls, member access, arrays, objects, templates, regular
expressions, legacy octal literals and escapes, `\u` escapes in identifiers,
and string escapes that produce lone surrogates.

Known deviations:

- Identifier characters are approximated with Unicode alphabetic and
  alphanumeric classes instead of ID_Start/ID_Continue.
- Nesting deeper than 256 levels is a RangeError.

## Architectural invariants

See [architecture.md](architecture.md#architectural-invariants).

## Planned post-MVP work

Not started, in rough order:

1. Specification Completion Records (`[[Type]]`, `[[Value]]`, `[[Target]]`)
   with UpdateEmpty and finally-block replacement, together with
   try/catch/finally.
2. Declarations, assignment, lexical environments, execution-context stack.
3. Objects, arrays, functions, closures, classes and prototypes; GC.
4. Promises, jobs, modules.
5. Host-integration work in an embedding (Node-API hosting, `.node` loading
   inside Alloy). That work lives outside this crate.
