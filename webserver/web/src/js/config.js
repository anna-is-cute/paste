/* global hljs:false, CodeSass:false */

(function() {
  function codeFlaskSucksHighlight(editor) {
    hljs.highlightBlock(editor.elCode);
    // remove the extra classes hljs adds without asking
    for (const clazz of editor.elCode.classList) {
      if (clazz !== 'hljs' && clazz !== 'codeflask__code' && !clazz.startsWith('language-')) {
        editor.elCode.classList.remove(clazz);
      }
    }
  }

  /**
   * Create an editor.
   *
   * @param {HTMLElement} el The element to convert into an editor.
   */
  function setUpEditor(el) {
    const div = document.createElement('div');

    div.style.height = '400px';

    const editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'toml',
    });

    editor.elTextarea.name = el.name;

    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';

    editor.setHighlightCallback(codeFlaskSucksHighlight);

    editor.updateCode(el.getAttribute('value'));
    editor.createLineNumbers(); // TODO: fix this in codesass

    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  setUpEditor(document.querySelector('textarea[name = config]'));
})();
