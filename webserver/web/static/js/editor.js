var paste_num = 0;
var paste_editors = {};

(function() {
  /**
   * Create the upload array for handling multiple files.
   */
  function createUpload() {
    var files = [];
    for (const editor of Object.values(paste_editors)) {
      const file = {
        'name': editor.container.parentElement.parentElement.parentElement.querySelector('input[name=file_name]').value,
        'content': editor.getValue()
      };
      const id = editor.container.parentElement.parentElement.parentElement.querySelector('input[name=id]');
      if (id !== null) {
        file['id'] = id.value;
      }
      files.push(file);
    }
    return files;
  }

  /**
   * Create an editor.
   *
   * @param {HTMLElement} parent The file container.
   * @param {HTMLElement} el The element to convert into an editor.
   */
  function setUpEditor(parent, el) {
    const modelist = ace.require('ace/ext/modelist');

    var theme;
    if (localStorage.getItem('style') === 'dark') {
      theme = 'ace/theme/idle_fingers';
    } else {
      theme = 'ace/theme/tomorrow';
    }

    const editor = ace.edit(el);

    editor.setTheme(theme);

    const hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.container.insertAdjacentElement('afterend', hidden);

    editor.setOptions({
      'maxLines': 25,
      'minLines': 25,
    });

    const name_input = parent.querySelector('input[name=file_name]');
    name_input.addEventListener('input', function(e) {
      const mode = modelist.getModeForPath(e.target.value).mode;
      editor.session.setMode(mode);
    });

    if (name_input.value !== '') {
      const mode = modelist.getModeForPath(name_input.value).mode;
      editor.session.setMode(mode);
    }

    const to_delete = paste_num;
    parent.querySelector('button[name=delete_button]').addEventListener('click', function() {
      removeFile(to_delete);
    });

    parent.querySelector('div[name=name_field]').classList.add('is-grouped');

    paste_editors[paste_num] = editor;
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
