'use strict';

/* global moment */

(function () {
  function updateTime(elem) {
    var ts = elem.dataset.timestamp;
    if (ts === undefined) {
      return;
    }

    var m = moment.utc(ts).local();

    elem.innerHTML = m.fromNow();
    elem.title = m.format('LLL');
  }

  function updateAllTimes() {
    var _iteratorNormalCompletion = true;
    var _didIteratorError = false;
    var _iteratorError = undefined;

    try {
      for (var _iterator = document.getElementsByClassName('timestamp')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
        var ts = _step.value;

        updateTime(ts);
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
  }

  (function () {
    if (navigator.languages) {
      moment.locale(navigator.languages);
    }

    updateAllTimes();

    setInterval(updateAllTimes, 60 * 1000);
  })();
})();
//# sourceMappingURL=timestamps.js.map