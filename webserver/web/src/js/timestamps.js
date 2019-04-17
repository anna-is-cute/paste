(function() {
  function updateTime(elem) {
    const ts = elem.dataset.timestamp;
    if (ts === undefined) {
      return;
    }

    const date = new Date(ts);
    const now = new Date();

    var difference = date - now;
    difference /= 1000;

    var seconds = difference % 60;
    difference /= 60;

    const minutes = difference % 60;
    difference /= 60;

    const hours = difference % 24;
    difference /= 24;

    const days = difference % 7;
    difference /= 7;

    const weeks = difference % 4;
    difference /= 4;

    const months = difference % 12;
    difference /= 12;

    const years = difference;

    const rtf = new Intl.RelativeTimeFormat();
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

    const title = new Intl.DateTimeFormat(undefined, {
      day: 'numeric',
      month: 'long',
      year: 'numeric',
      hour: 'numeric',
      minute: 'numeric',
    }).format(date);

    elem.innerHTML = val;
    elem.title = title;
  }

  function updateAllTimes() {
    for (const ts of document.getElementsByClassName('timestamp')) {
      updateTime(ts);
    }
  }

  (function() {
    updateAllTimes();

    setInterval(updateAllTimes, 60 * 1000);
  })();
})();
