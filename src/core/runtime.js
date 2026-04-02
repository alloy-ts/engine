(function() {
  var global = globalThis;

  // Internal bridge to the host
  var hostCall = global.alloy_host_call;
  // delete global.alloy_host_call; // keep it for now if needed

  var Alloy = {
    call: function(name, args) {
      // Direct call to registered native functions via the bridge
      var fn = global[name];
      if (!fn) throw new Error('Native function not found: ' + name);
      var res = fn.apply(null, args);
      return res ? JSON.parse(res) : null;
    }
  };

  global.Alloy = Alloy;

  // --- Polyfills ---
  if (!global.TextEncoder) {
      var TE = function TextEncoder() {};
      TE.prototype.encode = function(s) {
          var arr = new Uint8Array(s.length);
          for (var i = 0; i < s.length; i++) arr[i] = s.charCodeAt(i);
          return arr;
      };
      global.TextEncoder = TE;
  }
  if (!global.TextDecoder) {
      var TD = function TextDecoder() {};
      TD.prototype.decode = function(arr) {
          var s = '';
          for (var i = 0; i < arr.length; i++) s += String.fromCharCode(arr[i]);
          return s;
      };
      global.TextDecoder = TD;
  }

  // Simple Promise polyfill for synchronous execution in MQJS
  if (!global.Promise) {
      var P = function Promise(executor) {
          var self = this;
          this.state = 'pending';
          this.value = undefined;
          this.callbacks = [];

          function resolve(val) {
              if (self.state !== 'pending') return;
              self.state = 'fulfilled';
              self.value = val;
              for (var i = 0; i < self.callbacks.length; i++) {
                  self.callbacks[i].onFulfilled(val);
              }
          }
          function reject(reason) {
              if (self.state !== 'pending') return;
              self.state = 'rejected';
              self.value = reason;
              for (var i = 0; i < self.callbacks.length; i++) {
                  self.callbacks[i].onRejected(reason);
              }
          }
          try {
              executor(resolve, reject);
          } catch (e) {
              reject(e);
          }
      };
      P.prototype.then = function(onFulfilled, onRejected) {
          var self = this;
          return new P(function(resolve, reject) {
              function handle() {
                  try {
                      if (self.state === 'fulfilled') {
                          var res = onFulfilled ? onFulfilled(self.value) : self.value;
                          if (res instanceof P) res.then(resolve, reject);
                          else resolve(res);
                      } else {
                          var res = onRejected ? onRejected(self.value) : self.value;
                          if (res instanceof P) res.then(resolve, reject);
                          else reject(res);
                      }
                  } catch (e) {
                      reject(e);
                  }
              }
              if (self.state === 'pending') {
                  self.callbacks.push({ onFulfilled: handle, onRejected: handle });
              } else {
                  handle();
              }
          });
      };
      P.resolve = function(v) { return new P(function(r) { r(v); }); };
      global.Promise = P;
  }

  // --- Streams API ---

  var RSDC = function ReadableStreamDefaultController(stream) {
    this._stream = stream;
  };
  RSDC.prototype.enqueue = function(chunk) {
    if (this._stream._state !== 'readable') throw new Error('Stream not readable');
    this._stream._queue.push(chunk);
    this._stream._processPull();
  };
  RSDC.prototype.close = function() {
    this._stream._state = 'closed';
    this._stream._processPull();
  };
  RSDC.prototype.error = function(e) {
    this._stream._state = 'errored';
    this._stream._error = e;
  };

  var RDDC = function ReadableStreamDirectController(stream) {
    this._stream = stream;
  };
  RDDC.prototype.write = function(chunk) {
    if (this._stream._state !== 'readable') throw new Error('Stream not readable');
    if (this._stream._reader) {
      this._stream._reader._resolveNext({ value: chunk, done: false });
    } else {
      this._stream._queue.push(chunk);
    }
  };
  RDDC.prototype.close = function() {
    this._stream._state = 'closed';
    if (this._stream._reader) {
      this._stream._reader._resolveNext({ value: undefined, done: true });
    }
  };

  var RS = function ReadableStream(underlyingSource) {
    if (!underlyingSource) underlyingSource = {};
    this._state = 'readable';
    this._queue = [];
    this._underlyingSource = underlyingSource;
    this._reader = null;
    this._type = underlyingSource.type || 'default';

    var controller = this._type === 'direct'
      ? new RDDC(this)
      : new RSDC(this);

    if (underlyingSource.start) {
      underlyingSource.start(controller);
    }
  };

  Object.defineProperty(RS.prototype, 'locked', {
    get: function() { return this._reader !== null; }
  });

  RS.prototype.getReader = function() {
    if (this.locked) throw new Error('Stream is locked');
    this._reader = new RSDR(this);
    return this._reader;
  };

  RS.prototype._processPull = function() {
    if (!this._reader || !this._reader._pendingRead) return;
    if (this._queue.length > 0) {
      var chunk = this._queue.shift();
      this._reader._resolveNext({ value: chunk, done: false });
    } else if (this._state === 'closed') {
      this._reader._resolveNext({ value: undefined, done: true });
    } else if (this._state === 'errored') {
      this._reader._rejectNext(this._error);
    } else if (this._underlyingSource.pull) {
      var controller = this._type === 'direct'
          ? new RDDC(this)
          : new RSDC(this);
      this._underlyingSource.pull(controller);
    }
  };

  var RSDR = function ReadableStreamDefaultReader(stream) {
    this._stream = stream;
    this._pendingRead = null;
  };

  RSDR.prototype.read = function() {
    if (this._pendingRead) throw new Error('Read already in progress');
    var self = this;
    return new Promise(function(resolve, reject) {
      self._pendingRead = { resolve: resolve, reject: reject };
      self._stream._processPull();
    });
  };

  RSDR.prototype._resolveNext = function(result) {
    var read = this._pendingRead;
    this._pendingRead = null;
    if (read) read.resolve(result);
  };

  RSDR.prototype._rejectNext = function(error) {
      var read = this._pendingRead;
      this._pendingRead = null;
      if (read) read.reject(error);
  };

  RSDR.prototype.releaseLock = function() {
    this._stream._reader = null;
    this._stream = null;
  };

  global.ReadableStream = RS;

  var WS = function WritableStream(underlyingSink) {
      this._underlyingSink = underlyingSink || {};
      this._state = 'writable';
  };
  WS.prototype.getWriter = function() {
      return new WSDW(this);
  };

  var WSDW = function WritableStreamDefaultWriter(stream) {
      this._stream = stream;
  };
  WSDW.prototype.write = function(chunk) {
      if (this._stream._underlyingSink.write) {
          return Promise.resolve(this._stream._underlyingSink.write(chunk));
      }
      return Promise.resolve();
  };
  WSDW.prototype.close = function() {
      var self = this;
      var p = this._stream._underlyingSink.close ? Promise.resolve(this._stream._underlyingSink.close()) : Promise.resolve();
      return p.then(function() {
          self._stream._state = 'closed';
      });
  };

  global.WritableStream = WS;

  // --- Binary Data API ---

  var B = function Buffer(data, encodingOrOffset, length) {
      var b;
      if (typeof data === 'number') {
          b = new Uint8Array(data);
      } else if (typeof data === 'string') {
          var bytes = new TextEncoder().encode(data);
          b = new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      } else if (data instanceof ArrayBuffer) {
          b = new Uint8Array(data, encodingOrOffset, length);
      } else {
          b = new Uint8Array(data);
      }
      Object.setPrototypeOf(b, B.prototype);
      return b;
  };
  Object.setPrototypeOf(B.prototype, Uint8Array.prototype);
  Object.setPrototypeOf(B, Uint8Array);

  B.from = function(data, encodingOrOffset, length) {
      return new B(data, encodingOrOffset, length);
  };

  B.prototype.toString = function(encoding) {
      if (encoding === 'hex') {
          var res = '';
          for (var i = 0; i < this.length; i++) {
              var s = this[i].toString(16);
              res += s.length < 2 ? '0' + s : s;
          }
          return res;
      }
      if (encoding === 'base64') {
          var s = '';
          for (var i = 0; i < this.length; i++) s += String.fromCharCode(this[i]);
          return btoa(s);
      }
      return new TextDecoder().decode(this);
  };
  global.Buffer = B;

  var BL = function Blob(parts, options) {
      if (!parts) parts = [];
      if (!options) options = {};
      this._parts = parts.map(function(p) {
          if (typeof p === 'string') return new TextEncoder().encode(p);
          return p;
      });
      this.type = options.type || '';
      var size = 0;
      for (var i = 0; i < this._parts.length; i++) {
          size += this._parts[i].byteLength || this._parts[i].size || 0;
      }
      this.size = size;
  };

  BL.prototype.arrayBuffer = function() {
      var self = this;
      var res = new Uint8Array(this.size);
      var offset = 0;
      var chain = Promise.resolve();
      this._parts.forEach(function(part) {
          chain = chain.then(function() {
              var p = part instanceof BL ? part.arrayBuffer() : Promise.resolve(part);
              return p.then(function(buf) {
                  res.set(new Uint8Array(buf), offset);
                  offset += buf.byteLength;
              });
          });
      });
      return chain.then(function() { return res.buffer; });
  };

  BL.prototype.text = function() {
      return this.arrayBuffer().then(function(buf) {
          return new TextDecoder().decode(buf);
      });
  };

  BL.prototype.bytes = function() {
      return this.arrayBuffer().then(function(buf) {
          return new Uint8Array(buf);
      });
  };

  BL.prototype.stream = function() {
      var partIndex = 0;
      var parts = this._parts;
      return new ReadableStream({
          pull: function(controller) {
              if (partIndex < parts.length) {
                  var part = parts[partIndex++];
                  var p = part instanceof BL ? part.bytes() : Promise.resolve(part);
                  p.then(function(bytes) {
                      controller.enqueue(bytes);
                  });
              } else {
                  controller.close();
              }
          }
      });
  };
  global.Blob = BL;

  var F = function File(parts, name, options) {
      BL.call(this, parts, options);
      this.name = name;
      this.lastModified = (options && options.lastModified) || Date.now();
  };
  Object.setPrototypeOf(F.prototype, BL.prototype);
  global.File = F;

  var AF = function AlloyFile(path) {
      BL.call(this);
      this._path = path;
      var meta = global.fileStat(path);
      console.log('fileStat returned:', meta);
      var m = meta ? JSON.parse(meta) : { size: 0, lastModified: 0 };
      this.size = m.size;
      this.name = path.split('/').pop();
      this.lastModified = m.lastModified;
  };
  Object.setPrototypeOf(AF.prototype, BL.prototype);

  AF.prototype.stream = function() {
      var path = this._path;
      var size = this.size;
      var offset = 0;
      return new ReadableStream({
          type: 'direct',
          pull: function(controller) {
              if (offset < size) {
                  var res = global.fileRead(path, offset, 65536);
                  var chunk = res ? JSON.parse(res) : null;
                  if (chunk && chunk.length > 0) {
                      var bin = atob(chunk);
                      var bytes = new Uint8Array(bin.length);
                      for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
                      controller.write(bytes);
                      offset += bytes.length;
                      return;
                  }
              }
              controller.close();
          }
      });
  };

  AF.prototype.arrayBuffer = function() {
      return Alloy.readableStreamToArrayBuffer(this.stream());
  };
  Alloy.file = function(path) { return new AF(path); };

  var ABS = function ArrayBufferSink() {};
  ABS.prototype.start = function(options) {
      if (!options) options = {};
      this._asUint8Array = options.asUint8Array || false;
      this._stream = options.stream || false;
      this._buffer = new Uint8Array(options.highWaterMark || 1024);
      this._offset = 0;
  };
  ABS.prototype.write = function(chunk) {
      var bytes;
      if (typeof chunk === 'string') bytes = new TextEncoder().encode(chunk);
      else if (chunk instanceof ArrayBuffer) bytes = new Uint8Array(chunk);
      else bytes = new Uint8Array(chunk.buffer || chunk);

      if (this._offset + bytes.length > this._buffer.length) {
          var newBuf = new Uint8Array(Math.max(this._buffer.length * 2, this._offset + bytes.length));
          for (var i = 0; i < this._buffer.length; i++) newBuf[i] = this._buffer[i];
          this._buffer = newBuf;
      }
      for (var j = 0; j < bytes.length; j++) this._buffer[this._offset + j] = bytes[j];
      this._offset += bytes.length;
      return bytes.length;
  };
  ABS.prototype.flush = function() {
      var data = new Uint8Array(this._offset);
      for (var i = 0; i < this._offset; i++) data[i] = this._buffer[i];
      if (this._stream) {
          this._offset = 0;
      }
      return this._asUint8Array ? data : data.buffer;
  };
  ABS.prototype.end = function() {
      var data = this.flush();
      this._buffer = null;
      return data;
  };
  Alloy.ArrayBufferSink = ABS;

  // --- Conversion Utilities ---

  Alloy.readableStreamToArrayBuffer = function(stream) {
      return Alloy.readableStreamToBytes(stream).then(function(bytes) {
          return bytes.buffer;
      });
  };

  Alloy.readableStreamToBytes = function(stream) {
      var reader = stream.getReader();
      var chunks = [];
      var totalLength = 0;

      return new Promise(function(resolve, reject) {
          function read() {
              reader.read().then(function(result) {
                  if (result.done) {
                      var res = new Uint8Array(totalLength);
                      var offset = 0;
                      for (var i = 0; i < chunks.length; i++) {
                          res.set(chunks[i], offset);
                          offset += chunks[i].length;
                      }
                      resolve(res);
                      return;
                  }
                  var chunk = result.value;
                  var bytes = typeof chunk === 'string' ? new TextEncoder().encode(chunk) : new Uint8Array(chunk.buffer || chunk);
                  chunks.push(bytes);
                  totalLength += bytes.length;
                  read();
              }, reject);
          }
          read();
      });
  };

  Alloy.readableStreamToText = function(stream) {
      return Alloy.readableStreamToBytes(stream).then(function(bytes) {
          return new TextDecoder().decode(bytes);
      });
  };

  Alloy.readableStreamToArray = function(stream) {
      var reader = stream.getReader();
      var chunks = [];
      return new Promise(function(resolve, reject) {
          function read() {
              reader.read().then(function(result) {
                  if (result.done) {
                      resolve(chunks);
                      return;
                  }
                  chunks.push(result.value);
                  read();
              }, reject);
          }
          read();
      });
  };

  var R = function Response(body, options) {
      if (body && body.getReader) { // Simple stream detection
          this._stream = body;
      } else {
          this._stream = new ReadableStream({
              start: function(c) {
                  if (body) c.enqueue(body);
                  c.close();
              }
          });
      }
  };
  R.prototype.arrayBuffer = function() { return Alloy.readableStreamToArrayBuffer(this._stream); };
  R.prototype.text = function() { return Alloy.readableStreamToText(this._stream); };
  R.prototype.bytes = function() { return Alloy.readableStreamToBytes(this._stream); };
  R.prototype.blob = function() {
      return this.bytes().then(function(b) { return new BL([b]); });
  };
  global.Response = R;

})();
