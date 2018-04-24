for (var pre of document.getElementsByTagName('pre')) {
  if (pre.id === '') {
    continue;
  }
  var title = document.getElementById(pre.id + '-title');
  if (title === null) {
    continue;
  }
  var suffix = title.innerText.trim().split('.').pop();
  if (hljs.getLanguage(suffix) === undefined) {
    continue;
  }
  pre.classList.add('language-' + suffix);
  hljs.highlightBlock(pre);
}
