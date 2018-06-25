!(function(n, e) {
    "use strict";

    function r(t) {
        "complete" === e.readyState
            ? l(t)
            : n.addEventListener("DOMContentLoaded", function() {
                l(t);
            });
    }

    function l(t) {
        try {
            var r = e.querySelectorAll("code.hljs");
            for (var l in r) r.hasOwnProperty(l) && i(r[l], t);
        } catch (o) {
            n.console.error("LineNumbers error: ", o);
        }
    }

    function i(n, e) {
        if ("object" == typeof n) {
            e = e || {
                singleLine: !1
            };
            var t = e.singleLine ? 0 : 1;
            var prefix = n.dataset.lnPrefix;
            if (prefix) {
                prefix += "-";
            } else {
                prefix = "";
            }
            u(function() {
                s(n), (n.innerHTML = o(n.innerHTML, t, prefix));
            });
        }
    }

    function o(n, e, prefix) {
        var t = c(n);
        if ("" === t[t.length - 1].trim() && t.pop(), t.length > e) {
            for (var r = "", l = 0, i = t.length; l < i; l++) {
                r += h(
                    '<tr><td class="{0}"><a class="{1} {2}" {3}="{5}" name="{7}l{5}" href="#{7}l{5}"></a></td><td class="{4}"><div class="{1}">{6}</div></td></tr>',
                    [v, g, m, j, p, l + 1, t[l].length > 0 ? t[l] : " ", prefix]
                );
            }
            return h('<table class="{0}">{1}</table>', [f, r]);
        }
        return n;
    }

    function s(n) {
        var e = n.childNodes;
        for (var t in e) {
            if (e.hasOwnProperty(t)) {
            var r = e[t];
            d(r.textContent) > 0 &&
                (r.childNodes.length > 0 ? s(r) : a(r.parentNode));
            }
        }
    }

    function a(n) {
        var e = n.className;
        if (/hljs-/.test(e)) {
            for (var t = c(n.innerHTML), r = 0, l = ""; r < t.length; r++)
            l += h('<span class="{0}">{1}</span>\n', [e, t[r]]);
            n.innerHTML = l.trim();
        }
    }

    function c(n) {
        return 0 === n.length ? [] : n.split(L);
    }

    function d(n) {
        return (n.trim().match(L) || []).length;
    }

    function u(e) {
        var k = function() {
            e();
            document.onreadystatechange = function() {
            var hash = document.location.hash;
            document.location.hash = "";
            document.location.hash = hash;
            };
        };
        n.setTimeout(k, 0);
    }

    function h(n, e) {
        return n.replace(/\{(\d+)\}/g, function(n, t) {
            return e[t] ? e[t] : n;
        });
    }
    var f = "hljs-ln",
        g = "hljs-ln-line",
        p = "hljs-ln-code",
        v = "hljs-ln-numbers",
        m = "hljs-ln-n",
        j = "data-line-number",
        L = /\r\n|\r|\n/g;
    n.hljs
        ? ((n.hljs.initLineNumbersOnLoad = r), (n.hljs.lineNumbersBlock = i))
        : n.console.error("highlight.js not detected!");
})(window, document);
