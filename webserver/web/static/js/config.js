"use strict";

/* global hljs:false, CodeSass:false */
(function () {
  function codeFlaskSucksHighlight(editor) {
    hljs.highlightBlock(editor.elCode); // remove the extra classes hljs adds without asking

    var _iteratorNormalCompletion = true;
    var _didIteratorError = false;
    var _iteratorError = undefined;

    try {
      for (var _iterator = editor.elCode.classList[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
        var clazz = _step.value;

        if (clazz !== 'hljs' && clazz !== 'codeflask__code' && !clazz.startsWith('language-')) {
          editor.elCode.classList.remove(clazz);
        }
      }
    } catch (err) {
      _didIteratorError = true;
      _iteratorError = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion && _iterator.return != null) {
          _iterator.return();
        }
      } finally {
        if (_didIteratorError) {
          throw _iteratorError;
        }
      }
    }
  }
  /**
   * Create an editor.
   *
   * @param {HTMLElement} el The element to convert into an editor.
   */


  function setUpEditor(el) {
    var div = document.createElement('div');
    div.style.height = '400px';
    var editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'toml'
    });
    editor.elTextarea.name = el.name;
    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';
    editor.setHighlightCallback(codeFlaskSucksHighlight);
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  setUpEditor(document.querySelector('textarea[name = config]'));
})();
//# sourceMappingURL=config.js.map