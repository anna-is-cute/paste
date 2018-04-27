(function() {
  function openModal() {
    document.getElementById('deletion_modal').classList.add('is-active');
  }

  function closeModal() {
    document.getElementById('deletion_modal').classList.remove('is-active');
  }

  for (var e of document.getElementsByClassName('opens-modal')) {
    e.onclick = openModal;
  }

  for (var e of document.getElementsByClassName('closes-modal')) {
    e.onclick = closeModal;
  }
})();
