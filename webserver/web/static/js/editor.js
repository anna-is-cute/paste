var paste_num = 0;
var paste_editors = {};

(function() {
  /**
   * Create the upload array for handling multiple files.
   */
  function createUpload() {
    var files = [];
    for (var editor of Object.values(paste_editors)) {
      var file = {
        'name': editor.container.parentElement.parentElement.parentElement.querySelector('input[name=file_name]').value,
        'content': editor.getValue()
      };
      var id = editor.container.parentElement.parentElement.parentElement.querySelector('input[name=id]');
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
    var modelist = ace.require('ace/ext/modelist');

    var theme;
    if (localStorage.getItem("style") === "dark") {
      theme = 'ace/theme/idle_fingers';
    } else {
      theme = 'ace/theme/tomorrow';
    }

    var editor = ace.edit(el);

    editor.setTheme(theme);

    var hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.container.insertAdjacentElement('afterend', hidden);

    editor.setOptions({
      "maxLines": 25,
      "minLines": 25,
    });

    parent.querySelector('input[name=file_name]').oninput = function(e) {
      var mode = modelist.getModeForPath(e.target.value).mode;
      editor.session.setMode(mode);
    };

    var to_delete = paste_num;
    parent.querySelector('button[name=delete_button]').onclick = function() {
      removeFile(to_delete);
    };

    parent.querySelector('div[name=name_field]').classList.add('is-grouped');

    paste_editors[paste_num] = editor;
  }

  function addFile() {
    // get the base file for cloning (should be invisible if JS is running)
    var base = document.getElementById('base_file');

    // deep clone the base
    var clone = base.cloneNode(true);

    // show the editor by removing the requires-no-js class that was on the base
    clone.classList.remove('requires-no-js');

    paste_num += 1;
    clone.id = 'file' + paste_num;

    // set up an editor for each textarea in the base (should only be one)
    for (var ta of clone.getElementsByTagName('textarea')) {
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

    var file = document.getElementById('file' + num);

    if (file === null) {
      return;
    }

    delete paste_editors[num];

    file.remove();

    updateButtons();
  }

  function updateButtons() {
    var enabled = Object.keys(paste_editors).length > 1;
    for (var button of document.getElementsByName('delete_button')) {
      if (enabled) {
        button.disabled = false;
      } else {
        button.disabled = true;
      }
    }
  }

  function createEditors() {
    for (var editor of document.querySelectorAll('textarea.editor')) {
      paste_num += 1;
      setUpEditor(editor.parentElement.parentElement.parentElement, editor);
    }
    updateButtons();
  }

  document.getElementById('add_file').onclick = addFile;

  document.getElementById('paste_upload').onsubmit = function() {
    var input = document.createElement('input');
    input.type = 'hidden';
    input.value = JSON.stringify(createUpload());
    input.name = 'upload_json';

    this.appendChild(input);
  };

  // create any initial editors
  createEditors();

  // add an initial file if necessary
  if (Object.keys(paste_editors).length === 0) {
    addFile();
  }
})();
