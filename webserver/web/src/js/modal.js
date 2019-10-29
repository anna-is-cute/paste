(function() {
  function convertLinks() {
    // the links will all be linked to a modal but have a href
    for (const a of document.querySelectorAll('a[href][data-modal-id]')) {
      // create a button
      const button = document.createElement('button');
      // copy all attributes but href
      for (const attr of a.attributes) {
        if (attr.name === 'href') {
          continue;
        }
        button.setAttribute(attr.name, attr.value);
      }
      // fill the button with the link's children
      for (const child of a.children) {
        button.appendChild(child);
      }
      // make sure not to accidentally submit a form
      button.setAttribute('type', 'button');
      // replace the link
      a.replaceWith(button);
    }
  }

  // convert any links that should become modals if JS is enabled
  convertLinks();

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
