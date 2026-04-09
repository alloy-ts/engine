(function(){
    globalThis.fetch = function(url, options) {
        return new Promise(function(resolve, reject) {
            try {
                var res = nativeInvokeWithCap("fetch", JSON.stringify({url:url}), "web:fetch");
                var data = JSON.parse(res);
                resolve({ status: data.status, text: function(){ return Promise.resolve(data.body); } });
            } catch(e) { reject(e); }
        });
    };
})();
