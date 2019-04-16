"use strict";

(function () {
  function updateTime(elem) {
    var ts = elem.dataset.timestamp;

    if (ts === undefined) {
      return;
    }

    var date = new Date(ts);
    var now = new Date();
    var difference = date - now;
    difference /= 1000;
    var seconds = difference % 60;
    difference /= 60;
    var minutes = difference % 60;
    difference /= 60;
    var hours = difference % 60;
    difference /= 60;
    var days = difference % 24;
    difference /= 24;
    var weeks = difference % 7;
    difference /= 7;
    var months = difference % 4;
    difference /= 4;
    var years = difference % 12;
    var rtf = new Intl.RelativeTimeFormat();
    var val, period;

    if (years <= -1) {
      val = years;
      period = 'years';
    } else if (months <= -1) {
      val = months;
      period = 'months';
    } else if (weeks <= -1) {
      val = weeks;
      period = 'weeks';
    } else if (days <= -1) {
      val = days;
      period = 'days';
    } else if (hours <= -1) {
      val = hours;
      period = 'hours';
    } else if (minutes <= -1) {
      val = minutes;
      period = 'minutes';
    } else {
      if (seconds > -1) {
        seconds = -1;
      }

      val = seconds;
      period = 'seconds';
    }

    val = rtf.format(Math.floor(val), period);
    var title = new Intl.DateTimeFormat(undefined, {
      day: 'numeric',
      month: 'long',
      year: 'numeric',
      hour: 'numeric',
      minute: 'numeric'
    }).format(date);
    elem.innerHTML = val;
    elem.title = title;
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
        if (!_iteratorNormalCompletion && _iterator["return"] != null) {
          _iterator["return"]();
        }
      } finally {
        if (_didIteratorError) {
          throw _iteratorError;
        }
      }
    }
  }

  (function () {
    updateAllTimes();
    setInterval(updateAllTimes, 60 * 1000);
  })();
})();
//# sourceMappingURL=timestamps.js.map