(function() {
  for (var button of document.querySelectorAll('.message > .message-header > .delete')) {
    button.onclick = function() {
      this.parentElement.parentElement.remove();
    };
  }
  for (var button of document.querySelectorAll('.notification > .delete')) {
    button.onclick = function() {
      this.parentElement.remove();
    };
  }
})();
