var paste_editor = (function() {
  var modelist = ace.require('ace/ext/modelist');

  var theme;
  if (localStorage.getItem("style") === "dark") {
    theme = 'ace/theme/idle_fingers';
  } else {
    theme = 'ace/theme/tomorrow';
  }

  var editor = ace.edit('editor');

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

  document.getElementById('file_name').oninput = function(e) {
    var mode = modelist.getModeForPath(e.target.value).mode;
    editor.session.setMode(mode);
  };
  document.getElementById('paste_upload').onsubmit = function() {
    document.getElementById('hidden_content').value = editor.session.getValue();
  };

  return editor;
})();
