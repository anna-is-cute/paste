/* global hljs:false, CodeSass:false */

let pasteNum = 0;
const pasteEditors = {};

(function() {
  /**
   * Create the upload array for handling multiple files.
   */
  function createUpload() {
    function getLanguage(parent) {
      const lang = parent.querySelector('select[name=file_language]').value;
      if (lang === '') {
        return null;
      }
      return lang;
    }

    const files = [];
    for (const editor of Object.values(pasteEditors)) {
      const parent = editor.editorRoot.parentElement.parentElement.parentElement;
      const file = {
        'name': parent.querySelector('input[name=file_name]').value,
        'language': getLanguage(parent),
        'content': editor.getCode(),
      };
      const id = editor.editorRoot.parentElement.parentElement.parentElement.querySelector('input[name=id]');
      if (id !== null) {
        file.id = id.value;
      }
      files.push(file);
    }
    return files;
  }

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
   * @param {HTMLElement} parent The file container.
   * @param {HTMLElement} el The element to convert into an editor.
   */
  function setUpEditor(parent, el) {
    const div = document.createElement('div');

    div.style.height = '400px';

    const editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'plaintext',
    });

    const hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.editorRoot.insertAdjacentElement('afterend', hidden);

    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';

    editor.setHighlightCallback(codeFlaskSucksHighlight);

    const nameInput = parent.querySelector('input[name=file_name]');
    const langInput = parent.querySelector('select[name=file_language]');

    function updateLanguage() {
      let suffix;
      if (langInput.value !== '') {
        suffix = langInput.value;
      } else if (nameInput.value !== '') {
        suffix = nameInput.value.split('.').pop();
      }
      const lang = hljs.getLanguage(suffix) !== undefined ? suffix : 'plaintext';
      editor.updateLanguage(lang);
      editor.updateCode(editor.code);
    }

    nameInput.addEventListener('input', updateLanguage);
    langInput.addEventListener('change', updateLanguage);

    updateLanguage();
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    const toDelete = pasteNum;
    parent
      .querySelector('button[name=delete_button]')
      .addEventListener('click', () => removeFile(toDelete));

    pasteEditors[pasteNum] = editor;

    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  function addFile() {
    // get the base file for cloning (should be invisible if JS is running)
    const base = document.getElementById('base_file');

    // deep clone the base
    const clone = base.cloneNode(true);

    // show the editor by removing the requires-no-js class that was on the base
    clone.classList.remove('requires-no-js');

    pasteNum += 1;
    clone.id = `file${pasteNum}`;

    // set up an editor for each textarea in the base (should only be one)
    for (const ta of clone.getElementsByTagName('textarea')) {
      setUpEditor(clone, ta);
    }

    // add the editor to the dom
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

    const file = document.getElementById(`file${num}`);

    if (file === null) {
      return;
    }

    delete pasteEditors[num];

    file.remove();

    updateButtons();
  }

  function updateButtons() {
    const enabled = Object.keys(pasteEditors).length > 1;
    for (const button of document.getElementsByName('delete_button')) {
      if (enabled) {
        button.disabled = false;
      } else {
        button.disabled = true;
      }
    }
  }

  function createEditors() {
    for (const editor of document.querySelectorAll('textarea.editor')) {
      pasteNum += 1;
      setUpEditor(editor.parentElement.parentElement.parentElement, editor);
    }
    updateButtons();
  }

  document.getElementById('add_file').addEventListener('click', addFile);

  document.getElementById('paste_upload').addEventListener('submit', e => {
    const input = document.createElement('input');
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
