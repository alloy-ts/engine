const test = require('node:test');
const assert = require('node:assert');
const { add, Engine, Parser, Compiler, Context, Resolver } = require('./dist/index.js');

test('CJS require: add', () => {
  assert.strictEqual(add(2, 3), 5);
});

test('CJS require: Engine', () => {
  const engine = new Engine();
  assert.strictEqual(engine.create_int32(42), 42);
  assert.strictEqual(engine.create_string('hello'), 'hello');
});

test('CJS require: Engine error handling', () => {
  const engine = new Engine();
  assert.throws(() => engine.throw_error('custom error', 'ERR_CUSTOM'), {
    message: 'custom error',
  });
  assert.throws(() => engine.throw_range_error('range error', 'ERR_RANGE'), RangeError);
  assert.throws(() => engine.throw_type_error('type error', 'ERR_TYPE'), TypeError);
});

test('CJS require: Engine QuickJS settings & execPendingJob', () => {
  const engine = new Engine();
  engine.set_memory_limit(1024 * 1024);
  engine.run_gc();
  assert.strictEqual(engine.is_job_pending(), false);
  assert.strictEqual(engine.execPendingJob(), false);
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

test('CJS require: Compiler execute throws on invalid syntax', () => {
  const compiler = new Compiler();
  assert.throws(() => compiler.execute('const = ;', 'ts', 'app.ts'));
});

test('CJS require: Context eval', () => {
  const ctx = new Context();
  const res = ctx.eval('1 + 2');
  assert.strictEqual(res, '3');
});

test('CJS require: Resolver resolve', () => {
  const resolver = new Resolver();
  assert.strictEqual(resolver.resolve('/app/src/index.js', './utils.js'), '/app/src/utils.js');
});
