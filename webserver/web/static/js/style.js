"use strict";

(function () {
  function setActiveStyleSheet(title) {
    var _iteratorNormalCompletion = true;
    var _didIteratorError = false;
    var _iteratorError = undefined;

    try {
      for (var _iterator = document.getElementsByTagName('link')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
        var a = _step.value;

        if (a.getAttribute('rel').indexOf('style') !== -1 && a.getAttribute('title')) {
          a.disabled = true;

          if (a.getAttribute('title') === title) {
            a.disabled = false;
          }
        }
      }
    } catch (err) {
      _didIteratorError = true;
      _iteratorError = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion && _iterator.return != null) {
          _iterator.return();
        }
      } finally {
        if (_didIteratorError) {
          throw _iteratorError;
        }
      }
    }

    localStorage.setItem('style', getActiveStyleSheet());
  }

  function getActiveStyleSheet() {
    var _iteratorNormalCompletion2 = true;
    var _didIteratorError2 = false;
    var _iteratorError2 = undefined;

    try {
      for (var _iterator2 = document.getElementsByTagName('link')[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
        var a = _step2.value;

        if (a.getAttribute('rel').indexOf('style') !== -1 && a.getAttribute('title') && !a.disabled) {
          return a.getAttribute('title');
        }
      }
    } catch (err) {
      _didIteratorError2 = true;
      _iteratorError2 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion2 && _iterator2.return != null) {
          _iterator2.return();
        }
      } finally {
        if (_didIteratorError2) {
          throw _iteratorError2;
        }
      }
    }

    return null;
  }

  function getPreferredStyleSheet() {
    var _iteratorNormalCompletion3 = true;
    var _didIteratorError3 = false;
    var _iteratorError3 = undefined;

    try {
      for (var _iterator3 = document.getElementsByTagName('link')[Symbol.iterator](), _step3; !(_iteratorNormalCompletion3 = (_step3 = _iterator3.next()).done); _iteratorNormalCompletion3 = true) {
        var a = _step3.value;

        if (a.getAttribute('rel').indexOf('style') !== -1 && a.getAttribute('rel').indexOf('alt') === -1 && a.getAttribute('title')) {
          return a.getAttribute('title');
        }
      }
    } catch (err) {
      _didIteratorError3 = true;
      _iteratorError3 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion3 && _iterator3.return != null) {
          _iterator3.return();
        }
      } finally {
        if (_didIteratorError3) {
          throw _iteratorError3;
        }
      }
    }

    return null;
  }

  function swapTheme() {
    var next;

    if (getActiveStyleSheet() === 'dark') {
      next = 'light';
    } else {
      next = 'dark';
    }

    setActiveStyleSheet(next);
  }

  function loadSheet() {
    var style = localStorage.getItem('style');
    var title = style ? style : getPreferredStyleSheet();
    setActiveStyleSheet(title);
  }

  window.addEventListener('load', function () {
    loadSheet();
    document.getElementById('swap_theme').addEventListener('click', swapTheme);
  });
  window.addEventListener('unload', function () {
    var title = getActiveStyleSheet();
    this.localStorage.setItem('style', title);
  });
  loadSheet();
})();
//# sourceMappingURL=style.js.map