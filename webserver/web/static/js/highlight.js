'use strict';

function _toConsumableArray(arr) { if (Array.isArray(arr)) { for (var i = 0, arr2 = Array(arr.length); i < arr.length; i++) { arr2[i] = arr[i]; } return arr2; } else { return Array.from(arr); } }

/* global hljs:false */

(function () {
  var _loop = function _loop(pre) {
    (function () {
      if (pre.id === '') {
        return;
      }
      var title = document.getElementById(pre.id + '-title');
      if (title === null) {
        return;
      }
      var suffix = void 0;
      if (pre.lang) {
        suffix = pre.lang;
      } else {
        suffix = title.innerText.trim().split('.').pop();
      }
      var classes = [];
      if (hljs.getLanguage(suffix) === undefined) {
        classes.push('no-highlight');
        classes.push('hljs');
      } else {
        classes.push('language-' + suffix);
      }
      var _iteratorNormalCompletion2 = true;
      var _didIteratorError2 = false;
      var _iteratorError2 = undefined;

      try {
        for (var _iterator2 = classes[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
          var clazz = _step2.value;

          pre.classList.add(clazz);
        }
      } catch (err) {
        _didIteratorError2 = true;
        _iteratorError2 = err;
      } finally {
        try {
          if (!_iteratorNormalCompletion2 && _iterator2.return) {
            _iterator2.return();
          }
        } finally {
          if (_didIteratorError2) {
            throw _iteratorError2;
          }
        }
      }
    })();

    var cont = [].concat(_toConsumableArray(pre.classList)).some(function (e) {
      return e === 'hljs' || e.startsWith('language-');
    });

    if (!cont) {
      return 'continue';
    }

    hljs.highlightBlock(pre);

    if (pre.classList.contains('file-source')) {
      hljs.lineNumbersBlock(pre, {
        singleLine: true
      });
    }
  };

  var _iteratorNormalCompletion = true;
  var _didIteratorError = false;
  var _iteratorError = undefined;

  try {
    for (var _iterator = document.getElementsByTagName('pre')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
      var pre = _step.value;

      var _ret = _loop(pre);

      if (_ret === 'continue') continue;
    }
  } catch (err) {
    _didIteratorError = true;
    _iteratorError = err;
  } finally {
    try {
      if (!_iteratorNormalCompletion && _iterator.return) {
        _iterator.return();
      }
    } finally {
      if (_didIteratorError) {
        throw _iteratorError;
      }
    }
  }
})();
//# sourceMappingURL=highlight.js.map