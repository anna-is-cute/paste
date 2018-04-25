function setActiveStyleSheet(title) {
  for (var a of document.getElementsByTagName("link")) {
    if (a.getAttribute("rel").indexOf("style") != -1 && a.getAttribute("title")) {
      a.disabled = true;
      if (a.getAttribute("title") === title) {
        a.disabled = false;
      }
    }
  }
  if (typeof paste_editor !== 'undefined') {
    if (title === "dark") {
      paste_editor.setTheme("ace/theme/idle_fingers");
    } else if (title === "light") {
      paste_editor.setTheme("ace/theme/tomorrow");
    }
  }
}

function getActiveStyleSheet() {
  for (var a of document.getElementsByTagName("link")) {
    if (a.getAttribute("rel").indexOf("style") != -1 && a.getAttribute("title") && !a.disabled) {
      return a.getAttribute("title");
    }
  }
  return null;
}

function getPreferredStyleSheet() {
  for (var a of document.getElementsByTagName("link")) {
    if (a.getAttribute("rel").indexOf("style") != -1 && a.getAttribute("rel").indexOf("alt") == -1 && a.getAttribute("title")) {
      return a.getAttribute("title");
    }
  }
  return null;
}

function swapTheme() {
  var next;
  if (getActiveStyleSheet() == "dark") {
    next = "light";
  } else {
    next = "dark";
  }
  setActiveStyleSheet(next);
}

(function() {
  window.onload = function(e) {
    var style = this.localStorage.getItem("style");
    var title = style ? style : getPreferredStyleSheet();
    setActiveStyleSheet(title);
  }

  window.onunload = function(e) {
    var title = getActiveStyleSheet();
    this.localStorage.setItem("style", title);
  }

  var style = localStorage.getItem("style");
  var title = style ? style : getPreferredStyleSheet();
  setActiveStyleSheet(title);
})();
