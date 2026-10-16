import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const native = require("../dist/index.js");

export function main(): string {
  return "Hello, world!";
}

export const add = native.add;

// Proxy Parser to bridge camelCase napi-rs exports with snake_case test expectations
export class Parser {
  private _inner: any;
  constructor(sourceText: string, sourceType?: string) {
    this._inner = new native.Parser(sourceText, sourceType);
  }

  parser(options?: any) {
    return this._inner.parser(options);
  }

  parse_expression(options?: any) {
    const res = this._inner.parseExpression(options);
    if (res instanceof Error) {
      throw res;
    }
    return res;
  }

  parseExpression(options?: any) {
    const res = this._inner.parseExpression(options);
    if (res instanceof Error) {
      throw res;
    }
    return res;
  }
}

// Proxy Engine to bridge camelCase napi-rs exports with snake_case test expectations
export class Engine {
  private _inner: any;
  constructor() {
    this._inner = new native.Engine();
  }

  is_disposed() {
    return this._inner.isDisposed();
  }

  create_array(len: number) {
    return this._inner.createArray(len);
  }

  create_string(value: string) {
    return this._inner.createString(value);
  }

  create_int32(int: number) {
    return this._inner.createInt32(int);
  }

  create_double(value: number) {
    return this._inner.createDouble(value);
  }

  has_global(name: string) {
    return this._inner.hasGlobal(name);
  }

  get_napi_version() {
    return this._inner.getNapiVersion();
  }

  make_object() {
    return this._inner.makeObject();
  }

  set_instance_data(value: string) {
    return this._inner.setInstanceData(value);
  }

  get_instance_data() {
    return this._inner.getInstanceData();
  }
}
