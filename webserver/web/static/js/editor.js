'use strict';

/* global hljs:false, CodeSass:false */

var pasteNum = 0;
var pasteEditors = {};

(function () {
  /**
   * Create the upload array for handling multiple files.
   *
   * @returns {[{name: string, language: string, content: string}]} Array of upload files.
   */
  function createUpload() {
    function getLanguage(parent) {
      var lang = parent.querySelector('select[name=file_language]').value;
      if (lang === '') {
        return null;
      }
      return lang;
    }

    var files = [];
    var _iteratorNormalCompletion = true;
    var _didIteratorError = false;
    var _iteratorError = undefined;

    try {
      for (var _iterator = Object.values(pasteEditors)[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
        var editor = _step.value;

        var parent = editor.editorRoot.parentElement.parentElement.parentElement;
        var file = {
          'name': parent.querySelector('input[name=file_name]').value,
          'language': getLanguage(parent),
          'content': editor.getCode()
        };
        var id = editor.editorRoot.parentElement.parentElement.parentElement.querySelector('input[name=id]');
        if (id !== null) {
          file.id = id.value;
        }
        files.push(file);
      }
    } catch (err) {
      _didIteratorError = true;
      _iteratorError = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion && _iterator.return) {
          _iterator.return();
        }
      } finally {
        if (_didIteratorError) {
          throw _iteratorError;
        }
      }
    }

    return files;
  }

  function codeFlaskSucksHighlight(editor) {
    hljs.highlightBlock(editor.elCode);
    // remove the extra classes hljs adds without asking
    var _iteratorNormalCompletion2 = true;
    var _didIteratorError2 = false;
    var _iteratorError2 = undefined;

    try {
      for (var _iterator2 = editor.elCode.classList[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
        var clazz = _step2.value;

        if (clazz !== 'hljs' && clazz !== 'codeflask__code' && !clazz.startsWith('language-')) {
          editor.elCode.classList.remove(clazz);
        }
      }
    } catch (err) {
      _didIteratorError2 = true;
      _iteratorError2 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion2 && _iterator2.return) {
          _iterator2.return();
        }
      } finally {
        if (_didIteratorError2) {
          throw _iteratorError2;
        }
      }
    }
  }

  /**
   * Create an editor.
   *
   * @param {HTMLElement} parent The file container.
   * @param {HTMLElement} el The element to convert into an editor.
   */
  function setUpEditor(parent, el) {
    var div = document.createElement('div');

    div.style.height = '400px';

    var editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'plaintext'
    });

    var hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.editorRoot.insertAdjacentElement('afterend', hidden);

    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';

    editor.setHighlightCallback(codeFlaskSucksHighlight);

    var nameInput = parent.querySelector('input[name=file_name]');
    var langInput = parent.querySelector('select[name=file_language]');

    function updateLanguage() {
      var suffix = void 0;
      if (langInput.value !== '') {
        suffix = langInput.value;
      } else if (nameInput.value !== '') {
        suffix = nameInput.value.split('.').pop();
      }
      var lang = hljs.getLanguage(suffix) !== undefined ? suffix : 'plaintext';
      editor.updateLanguage(lang);
      editor.updateCode(editor.code);
    }

    nameInput.addEventListener('input', updateLanguage);
    langInput.addEventListener('change', updateLanguage);

    updateLanguage();
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    var toDelete = pasteNum;
    parent.querySelector('button[name=delete_button]').addEventListener('click', function () {
      return removeFile(toDelete);
    });

    pasteEditors[pasteNum] = editor;

    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  function addFile() {
    // get the base file for cloning (should be invisible if JS is running)
    var base = document.getElementById('base_file');

    // deep clone the base
    var clone = base.cloneNode(true);

    // show the editor by removing the requires-no-js class that was on the base
    clone.classList.remove('requires-no-js');

    pasteNum += 1;
    clone.id = 'file' + pasteNum;

    // set up an editor for each textarea in the base (should only be one)
    var _iteratorNormalCompletion3 = true;
    var _didIteratorError3 = false;
    var _iteratorError3 = undefined;

    try {
      for (var _iterator3 = clone.getElementsByTagName('textarea')[Symbol.iterator](), _step3; !(_iteratorNormalCompletion3 = (_step3 = _iterator3.next()).done); _iteratorNormalCompletion3 = true) {
        var ta = _step3.value;

        setUpEditor(clone, ta);
      }

      // add the editor to the dom
    } catch (err) {
      _didIteratorError3 = true;
      _iteratorError3 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion3 && _iterator3.return) {
          _iterator3.return();
        }
      } finally {
        if (_didIteratorError3) {
          throw _iteratorError3;
        }
      }
    }

    document.getElementById('end_of_files').insertAdjacentElement('beforebegin', clone);

    updateButtons();
  }

  /**
   * Remove a file. Will never remove the last file.
   *
   * @param {Number} num The number of the file to remove.
   */
  function removeFile(num) {
    if (Object.keys(pasteEditors).length === 1) {
      return;
    }

    var file = document.getElementById('file' + num);

    if (file === null) {
      return;
    }

    delete pasteEditors[num];

    file.remove();

    updateButtons();
  }

  function updateButtons() {
    var enabled = Object.keys(pasteEditors).length > 1;
    var _iteratorNormalCompletion4 = true;
    var _didIteratorError4 = false;
    var _iteratorError4 = undefined;

    try {
      for (var _iterator4 = document.getElementsByName('delete_button')[Symbol.iterator](), _step4; !(_iteratorNormalCompletion4 = (_step4 = _iterator4.next()).done); _iteratorNormalCompletion4 = true) {
        var button = _step4.value;

        if (enabled) {
          button.disabled = false;
        } else {
          button.disabled = true;
        }
      }
    } catch (err) {
      _didIteratorError4 = true;
      _iteratorError4 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion4 && _iterator4.return) {
          _iterator4.return();
        }
      } finally {
        if (_didIteratorError4) {
          throw _iteratorError4;
        }
      }
    }
  }

  function createEditors() {
    var _iteratorNormalCompletion5 = true;
    var _didIteratorError5 = false;
    var _iteratorError5 = undefined;

    try {
      for (var _iterator5 = document.querySelectorAll('textarea.editor')[Symbol.iterator](), _step5; !(_iteratorNormalCompletion5 = (_step5 = _iterator5.next()).done); _iteratorNormalCompletion5 = true) {
        var editor = _step5.value;

        pasteNum += 1;
        setUpEditor(editor.parentElement.parentElement.parentElement, editor);
      }
    } catch (err) {
      _didIteratorError5 = true;
      _iteratorError5 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion5 && _iterator5.return) {
          _iterator5.return();
        }
      } finally {
        if (_didIteratorError5) {
          throw _iteratorError5;
        }
      }
    }

    updateButtons();
  }

  document.getElementById('add_file').addEventListener('click', addFile);

  document.getElementById('paste_upload').addEventListener('submit', function (e) {
    var input = document.createElement('input');
    input.type = 'hidden';
    input.value = JSON.stringify(createUpload());
    input.name = 'upload_json';

    e.target.appendChild(input);
  });

  // create any initial editors
  createEditors();

  // add an initial file if necessary
  if (Object.keys(pasteEditors).length === 0) {
    addFile();
  }
})();
//# sourceMappingURL=editor.js.map