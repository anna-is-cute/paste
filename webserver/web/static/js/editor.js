var paste_num = 0;
var paste_editors = {};

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

    var files = [];
    for (const editor of Object.values(paste_editors)) {
      const parent = editor.editorRoot.parentElement.parentElement.parentElement;
      const file = {
        'name': parent.querySelector('input[name=file_name]').value,
        'language': getLanguage(parent),
        'content': editor.getCode(),
      };
      const id = editor.editorRoot.parentElement.parentElement.parentElement.querySelector('input[name=id]');
      if (id !== null) {
        file['id'] = id.value;
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

    editor.setHighlightCallback(function(ed) {
      codeFlaskSucksHighlight(ed);
    });

    const name_input = parent.querySelector('input[name=file_name]');
    const lang_input = parent.querySelector('select[name=file_language]');

    function updateLanguage() {
      var suffix;
      if (lang_input.value !== '') {
        suffix = lang_input.value;
      } else if (name_input.value !== '') {
        suffix = name_input.value.split('.').pop();
      }
      const lang = hljs.getLanguage(suffix) !== undefined ? suffix : 'plaintext';
      editor.updateLanguage(lang);
      editor.updateCode(editor.code);
    }

    name_input.addEventListener('input', updateLanguage);
    lang_input.addEventListener('change', updateLanguage);

    updateLanguage();
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    const to_delete = paste_num;
    parent.querySelector('button[name=delete_button]').addEventListener('click', function() {
      removeFile(to_delete);
    });

    paste_editors[paste_num] = editor;

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

    paste_num += 1;
    clone.id = 'file' + paste_num;

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
    if (Object.keys(paste_editors).length === 1) {
      return;
    }

    const file = document.getElementById('file' + num);

    if (file === null) {
      return;
    }

    delete paste_editors[num];

    file.remove();

    updateButtons();
  }

  function updateButtons() {
    const enabled = Object.keys(paste_editors).length > 1;
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
      paste_num += 1;
      setUpEditor(editor.parentElement.parentElement.parentElement, editor);
    }
    updateButtons();
  }

  document.getElementById('add_file').addEventListener('click', addFile);

  document.getElementById('paste_upload').addEventListener('submit', function() {
    const input = document.createElement('input');
    input.type = 'hidden';
    input.value = JSON.stringify(createUpload());
    input.name = 'upload_json';

    this.appendChild(input);
  });

  // create any initial editors
  createEditors();

  // add an initial file if necessary
  if (Object.keys(paste_editors).length === 0) {
    addFile();
  }
})();
