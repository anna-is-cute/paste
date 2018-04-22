function setActiveStyleSheet(title) {
  for (var a of document.getElementsByTagName("link")) {
    if (a.getAttribute("rel").indexOf("style") != -1 && a.getAttribute("title")) {
      a.disabled = true;
      if (a.getAttribute("title") === title) {
        a.disabled = false;
      }
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
