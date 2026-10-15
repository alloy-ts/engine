const test = require('node:test');
const assert = require('node:assert');
const { add, Engine, Parser, Compiler } = require('./dist/index.js');

test('CJS require: add', () => {
  assert.strictEqual(add(2, 3), 5);
});

test('CJS require: Engine', () => {
  const engine = new Engine();
  assert.strictEqual(engine.create_int32(42), 42);
  assert.strictEqual(engine.create_string('hello'), 'hello');
});

test('CJS require: Parser', () => {
  const parser = new Parser('const x = 1;');
  const res = parser.parser();
  assert.strictEqual(res.fatalError, false);
});

test('CJS require: Parser parse_expression throws on invalid', () => {
  const parser = new Parser('1 +');
  assert.throws(() => parser.parse_expression(undefined));
});

test('CJS require: Compiler execute', () => {
  const compiler = new Compiler();
  const output = compiler.execute('const x = 1;', 'ts', 'app.ts');
  assert.ok(typeof output === 'string');
});
