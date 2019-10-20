(function() {
  function openModal(e) {
    document.getElementById(e.dataset.modalId).classList.add('is-active');
  }

  function closeModal(e) {
    document.getElementById(e.dataset.modalId).classList.remove('is-active');
  }

  [...document.getElementsByClassName('opens-modal')].forEach(e => {
    e.addEventListener('click', () => openModal(e));
    if (e.tagName.toLowerCase() === 'button') {
      e.setAttribute('type', 'button');
    }
  });

  [...document.getElementsByClassName('closes-modal')].forEach(e => {
    e.addEventListener('click', () => closeModal(e));
    if (e.tagName.toLowerCase() === 'button') {
      e.setAttribute('type', 'button');
    }
  });
})();
