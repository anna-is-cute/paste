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
    console.log("difference: ".concat(difference));
    var seconds = difference % 60;
    console.log("seconds: ".concat(seconds));
    difference /= 60;
    var minutes = difference % 60;
    console.log("minutes: ".concat(minutes));
    difference /= 60;
    var hours = difference % 24;
    console.log("hours: ".concat(hours));
    difference /= 24;
    var days = difference % 7;
    console.log("days: ".concat(days));
    difference /= 7;
    var weeks = difference % 4;
    console.log("weeks: ".concat(weeks));
    difference /= 4;
    var months = difference % 12;
    console.log("months: ".concat(months));
    difference /= 12;
    var years = difference;
    console.log("years: ".concat(years));
    var rtf = new Intl.RelativeTimeFormat();
    var val, period;

    if (Math.abs(years) >= 1) {
      val = years;
      period = 'years';
    } else if (Math.abs(months) >= 1) {
      val = months;
      period = 'months';
    } else if (Math.abs(weeks) >= 1) {
      val = weeks;
      period = 'weeks';
    } else if (Math.abs(days) >= 1) {
      val = days;
      period = 'days';
    } else if (Math.abs(hours) >= 1) {
      val = hours;
      period = 'hours';
    } else if (Math.abs(minutes) >= 1) {
      val = minutes;
      period = 'minutes';
    } else {
      if (Math.abs(seconds) < 1) {
        seconds = difference < 0 ? -1 : 1;
      }

      val = seconds;
      period = 'seconds';
    }

    val = rtf.format(Math.trunc(val), period);
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