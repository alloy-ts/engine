// ES5 test
var val = 1 + 1;
if (typeof nativeInvokeWithCap === "function") {
    nativeInvokeWithCap("fetch", "{}", "web:fetch");
}
