"use strict";

function _toConsumableArray(arr) { return _arrayWithoutHoles(arr) || _iterableToArray(arr) || _nonIterableSpread(); }

function _nonIterableSpread() { throw new TypeError("Invalid attempt to spread non-iterable instance"); }

function _iterableToArray(iter) { if (Symbol.iterator in Object(iter) || Object.prototype.toString.call(iter) === "[object Arguments]") return Array.from(iter); }

function _arrayWithoutHoles(arr) { if (Array.isArray(arr)) { for (var i = 0, arr2 = new Array(arr.length); i < arr.length; i++) { arr2[i] = arr[i]; } return arr2; } }

/* global hljs:false */
(function () {
  function getSuffixFromName(name) {
    if (name === 'CMakeLists.txt') {
      return 'CMake';
    }

    return name.split('.').pop();
  }

  var _iteratorNormalCompletion = true;
  var _didIteratorError = false;
  var _iteratorError = undefined;

  try {
    var _loop = function _loop() {
      var pre = _step.value;

      (function () {
        if (pre.id === '') {
          return;
        }

        var title = document.getElementById("".concat(pre.id, "-title"));

        if (title === null) {
          return;
        }

        var suffix;

        if (pre.lang) {
          suffix = pre.lang;
        } else {
          suffix = getSuffixFromName(title.innerText.trim());
        }

        var classes = [];

        if (hljs.getLanguage(suffix) === undefined) {
          classes.push('no-highlight');
          classes.push('hljs');
        } else {
          classes.push("language-".concat(suffix));
        }

        for (var _i = 0, _classes = classes; _i < _classes.length; _i++) {
          var clazz = _classes[_i];
          pre.classList.add(clazz);
        }
      })();

      var cont = _toConsumableArray(pre.classList).some(function (e) {
        return e === 'hljs' || e.startsWith('language-');
      });

      if (!cont) {
        return "continue";
      }

      hljs.highlightBlock(pre);

      if (pre.classList.contains('file-source')) {
        hljs.lineNumbersBlock(pre, {
          singleLine: true
        });
      }
    };

    for (var _iterator = document.getElementsByTagName('pre')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
      var _ret = _loop();

      if (_ret === "continue") continue;
    }
  } catch (err) {
    _didIteratorError = true;
    _iteratorError = err;
  } finally {
    try {
      if (!_iteratorNormalCompletion && _iterator["return"] != null) {
        _iterator["return"]();
      }
    } finally {
      if (_didIteratorError) {
        throw _iteratorError;
      }
    }
  }
})();
//# sourceMappingURL=highlight.js.map