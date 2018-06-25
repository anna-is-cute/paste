(function() {
  function setActiveStyleSheet(title) {
    for (const a of document.getElementsByTagName('link')) {
      if (a.getAttribute('rel').indexOf('style') != -1 && a.getAttribute('title')) {
        a.disabled = true;
        if (a.getAttribute('title') === title) {
          a.disabled = false;
        }
      }
    }
    localStorage.setItem('style', getActiveStyleSheet());
  }

  function getActiveStyleSheet() {
    for (var a of document.getElementsByTagName('link')) {
      if (a.getAttribute('rel').indexOf('style') != -1 && a.getAttribute('title') && !a.disabled) {
        return a.getAttribute('title');
      }
    }
    return null;
  }

  function getPreferredStyleSheet() {
    for (var a of document.getElementsByTagName('link')) {
      if (a.getAttribute('rel').indexOf('style') != -1 && a.getAttribute('rel').indexOf('alt') == -1 && a.getAttribute('title')) {
        return a.getAttribute('title');
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
    const style = localStorage.getItem('style');
    const title = style ? style : getPreferredStyleSheet();
    setActiveStyleSheet(title);
  }

  window.addEventListener('load', function() {
    loadSheet();

    document.getElementById('swap_theme').addEventListener('click', swapTheme);
  });

  window.addEventListener('unload', function() {
    const title = getActiveStyleSheet();
    this.localStorage.setItem('style', title);
  });

  loadSheet();
})();
