(function() {
  for (const pre of document.getElementsByTagName('pre')) {
    if (pre.id === '') {
      continue;
    }
    const title = document.getElementById(pre.id + '-title');
    if (title === null) {
      continue;
    }
    const suffix = title.innerText.trim().split('.').pop();
    const classes = [];
    if (hljs.getLanguage(suffix) === undefined) {
      classes.push('no-highlight');
      classes.push('hljs');
    } else {
      classes.push('language-' + suffix);
    }
    for (const clazz of classes) {
      pre.classList.add(clazz);
    }
    hljs.highlightBlock(pre);
    hljs.lineNumbersBlock(pre);
  }
})();
