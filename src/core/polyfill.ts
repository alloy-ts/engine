export const ASYNC_POLYFILL = `
(function(g) {
  if (g.Promise) return;
  var tasks = [];
  g.Promise = function(exec) {
    var self = this;
    this.s = 0; // 0: pending, 1: resolved, 2: rejected
    this.v = undefined;
    this.h = [];
    function resolve(v) {
      if (self.s) return;
      self.s = 1; self.v = v;
      tasks.push(function() { self.h.forEach(function(cb) { cb(v); }); });
    }
    try { exec(resolve); } catch(e) { /* simplified */ }
  };
  g.Promise.prototype.then = function(cb) {
    if (this.s === 1) tasks.push(function() { cb(this.v); }.bind(this));
    else this.h.push(cb);
    return this;
  };
  g.__alloy_loop = function() {
    var t;
    while(tasks.length > 0) {
      t = tasks.shift();
      if (t) t();
    }
  };
})(globalThis);
`;
