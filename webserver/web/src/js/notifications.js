(function() {
  for (const button of document.querySelectorAll('.message > .message-header > .delete')) {
    button.addEventListener('click', e => e.currentTarget.parentElement.parentElement.remove());
  }
  for (const button of document.querySelectorAll('.notification > .delete')) {
    button.addEventListener('click', e => e.currentTarget.parentElement.remove());
  }
})();
