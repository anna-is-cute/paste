(function() {
  var modelist = ace.require('ace/ext/modelist');
  var editor = ace.edit('editor');
  var hidden = document.createElement('input');
  hidden.type = 'hidden';
  hidden.name = 'file_content';
  hidden.id = 'hidden_content';
  editor.container.insertAdjacentElement('afterend', hidden);
  editor.setOptions({
    "maxLines": 25,
    "minLines": 25,
  });
  editor.setTheme("ace/theme/idle_fingers");
  document.getElementById('file_name').oninput = function(e) {
    var mode = modelist.getModeForPath(e.target.value).mode;
    editor.session.setMode(mode);
  };
  document.getElementById('paste_upload').onsubmit = function() {
    document.getElementById('hidden_content').value = editor.session.getValue();
  };
})();
