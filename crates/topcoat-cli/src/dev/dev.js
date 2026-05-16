(function () {
  var url = new URL(document.currentScript.src);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = "/ws";
  url.search = "";
  url.hash = "";
  var wsUrl = url.toString();

  function connect() {
    var ws = new WebSocket(wsUrl);
    ws.onmessage = function (e) {
      if (e.data === "reload") window.location.reload();
    };
    ws.onclose = function () {
      setTimeout(function () {
        var retry = new WebSocket(wsUrl);
        retry.onopen = function () {
          retry.close();
          window.location.reload();
        };
        retry.onerror = function () {
          setTimeout(connect, 1000);
        };
      }, 500);
    };
  }
  connect();
})();
