# Engine plan

## Implemented

Language subset, following ECMA-262 semantics for these features. Scripts
are non-strict (sloppy) code.

- Values: undefined, null, boolean, number (f64), string.
- Literals: decimal numbers with fraction and exponent, `0x`/`0o`/`0b`
  integers; `'...'` and `"..."` strings with `\n \t \r \b \f \v \0 \' \" \\`,
  `\xHH`, `\uHHHH`, `\u{...}` (surrogate pairs combined), line continuations,
  and identity escapes; `true`, `false`, `null`.
- Declarations: `var`, `let` and `const` with comma-separated bindings and
  optional initializers (required for `const`). `let` is a keyword only
  before a binding identifier; otherwise it is an identifier, as in sloppy
  code.
  - `var` binds a property of the global object (hoisted, including from
    inside blocks, and initialized to undefined).
  - `let` / `const` bind in the global or block Declarative Environment
    Record, with a temporal dead zone: reading or assigning before the
    declaration runs is a ReferenceError.
  - Assigning a `const` is a TypeError.
- Blocks `{ ... }`, each with its own scope for `let` / `const`.
- Identifier references resolve through block scopes, then global
  `let` / `const`, then the global object. An unknown name is a
  ReferenceError (`x is not defined`). Seeded non-writable, non-configurable
  globals: `undefined`, `NaN`, `Infinity`.
- Assignment `=` and compound `+= -= *= /= %=` to identifiers (optionally
  parenthesized), right-associative. The target is resolved first; compound
  assignment reads it before evaluating the right side. Assigning an
  undeclared name creates a global; assigning a read-only global is ignored.
  Any other target is an early SyntaxError.
- Parentheses; unary `+ - !`; binary `* / %`, `+ -`, `< <= > >=`,
  `== != === !==` with ECMA-262 precedence and left associativity.
- Statements separated by `;` or by automatic semicolon insertion (before
  `}`, at end of input, or before a token on a new line that cannot continue
  the statement: `1\n+2` is one statement). Empty statements are allowed.
- Completion values follow UpdateEmpty: a Script's value is that of its
  last value-producing statement (declarations and empty blocks produce
  none), or undefined.
- Early errors, reported before any statement of the Script runs:
  duplicate `let` / `const` names in a Script or Block, a lexical name that
  is also a `var` name in the same Script or Block (including nested
  `var`s), `let` as a lexically bound name, a `const` without initializer,
  and an invalid assignment target. GlobalDeclarationInstantiation adds the
  checks against earlier Scripts in the same Realm (see
  [architecture.md](architecture.md#environments-and-bindings)).
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

Rejected with SyntaxError because they are outside the subset: other
reserved words (`typeof`, `if`, `function`, ...), `**`, shifts, logical and
bitwise operators and their assignment forms, `=>`, `++`/`--`, the comma
operator, calls, member access, arrays, objects, destructuring, templates,
regular expressions, directives (`"use strict"` is an ordinary expression),
legacy octal literals and escapes, `\u` escapes in identifiers, and string
escapes that produce lone surrogates.

Known deviations:

- Identifier characters are approximated with Unicode alphabetic and
  alphanumeric classes instead of ID_Start/ID_Continue.
- Nesting deeper than 256 levels is a RangeError.
- The global object is a simplified property map; see
  [architecture.md](architecture.md#deviations-from-ecma-262).

## Architectural invariants

See [architecture.md](architecture.md#architectural-invariants).

## Planned work

Not started, in rough order:

1. Control flow: `if`, loops, labels, `break` / `continue` (break and
   continue completions with `[[Target]]`), `throw`, `try` / `catch` /
   `finally` with finally-block completion replacement; `typeof` and the
   logical operators.
2. Objects, arrays, functions (return completions), closures, classes and
   prototypes; an execution-context stack and a real global object; GC.
3. Promises, jobs, modules.
4. Host-integration work beyond the NAPI facade (`.node` loading inside
   Alloy, passing objects across the host boundary).
