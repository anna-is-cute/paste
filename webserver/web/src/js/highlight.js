/* global hljs:false */

(function() {
  function getSuffixFromName(name) {
    if (name === 'CMakeLists.txt') {
      return 'cmake';
    } else if (name.toLowerCase().endsWith(".svg")) {
      return 'xml';
    }

    return name.split('.').pop();
  }

  for (const pre of document.getElementsByTagName('pre')) {
    (function() {
      if (pre.id === '') {
        return;
      }
      const title = document.getElementById(`${pre.id}-title`);
      if (title === null) {
        return;
      }
      let suffix;
      if (pre.lang) {
        suffix = pre.lang;
      } else {
        suffix = getSuffixFromName(title.innerText.trim());
      }
      const classes = [];
      if (hljs.getLanguage(suffix) === undefined) {
        classes.push('no-highlight');
        classes.push('hljs');
      } else {
        classes.push(`language-${suffix}`);
      }
      for (const clazz of classes) {
        pre.classList.add(clazz);
      }
    })();

    const cont = [...pre.classList].some(e => e === 'hljs' || e.startsWith('language-'));

    if (!cont) {
      continue;
    }

    hljs.highlightBlock(pre);

    if (pre.classList.contains('file-source')) {
      hljs.lineNumbersBlock(pre, {
        singleLine: true,
      });
    }
  }
})();
