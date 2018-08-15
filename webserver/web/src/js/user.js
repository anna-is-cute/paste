(function() {
  function getCheckboxes() {
    return document.getElementsByName('paste-delete');
  }

  function buttonVisibilityCheck() {
    const button = document.getElementById('delete-button');
    if ([...getCheckboxes()].some(x => x.checked)) {
      button.classList.remove('is-hidden');
    } else {
      button.classList.add('is-hidden');
    }
  }

  function getPastesToDelete() {
    return [...getCheckboxes()]
      .filter(x => x.checked)
      .map(x => x.dataset.pasteId);
  }

  function addInput(form) {
    const input = document.createElement('input');
    input.type = 'hidden';
    input.name = 'ids';
    input.value = JSON.stringify(getPastesToDelete());

    form.appendChild(input);
  }

  (function() {
    getCheckboxes().forEach(x => x.addEventListener('change', buttonVisibilityCheck));

    const form = document.getElementById('deletion_form');
    form.addEventListener('submit', () => addInput(form));
  })();
})();
