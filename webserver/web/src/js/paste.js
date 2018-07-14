(function() {
  function openModal() {
    document.getElementById('deletion_modal').classList.add('is-active');
  }

  function closeModal() {
    document.getElementById('deletion_modal').classList.remove('is-active');
  }

  [...document.getElementsByClassName('opens-modal')].forEach(e => e.addEventListener('click', openModal));

  [...document.getElementsByClassName('closes-modal')].forEach(e => e.addEventListener('click', closeModal));

  function swap(current, currentContent, next, nextContent) {
    current.classList.remove('is-active');
    next.classList.add('is-active');

    currentContent.classList.add('is-not-displayed');
    nextContent.classList.remove('is-not-displayed');
  }

  for (const tabsContainer of document.getElementsByClassName('paste-tabs-container')) {
    const fileId = tabsContainer.dataset.id;
    const tabLinks = document.getElementById(`${fileId}-tab-links`);

    const rendered = tabLinks.querySelector('.paste-rendered-tab');
    const renderedA = rendered.firstChild;

    const source = tabLinks.querySelector('.paste-source-tab');
    const sourceA = source.firstChild;

    const renderedContent = tabsContainer.querySelector('div.paste-rendered-content');
    const sourceContent = tabsContainer.querySelector('div.paste-source-content');

    renderedA.addEventListener('click', () => swap(source, sourceContent, rendered, renderedContent));
    sourceA.addEventListener('click', () => swap(rendered, renderedContent, source, sourceContent));
  }

  function getDeletionKeys() {
    let keys = localStorage.getItem('deletion_keys');

    if (keys === null) {
      keys = {};
    } else {
      keys = JSON.parse(keys);
    }

    return keys;
  }

  function setDeletionKeys(keys) {
    localStorage.setItem('deletion_keys', JSON.stringify(keys));
  }

  // check if the page is displaying a deletion key and add it to local storage
  (function() {
    const dkElem = document.getElementById('deletion_key');

    if (dkElem === null) {
      return;
    }

    const deletionKey = dkElem.innerText;

    const keys = getDeletionKeys();

    const pasteId = dkElem.dataset.pasteId;

    keys[pasteId] = {
      deletionKey,
      expires: new Date((new Date).getTime() + 30 * 24 * 60 * 60 * 1000),
    };

    setDeletionKeys(keys);
  })();

  // check if we have a deletion key for this paste and insert it
  (function() {
    const dkInput = document.getElementById('deletion_key_input');

    if (dkInput === null) {
      return;
    }

    const pasteId = dkInput.dataset.pasteId;

    const keys = getDeletionKeys();

    const key = keys[pasteId];

    if (key === undefined) {
      return;
    }

    dkInput.value = key.deletionKey;

    // add a listener for form submit to remove key from local storage
    const deletionForm = document.getElementById('deletion_form');

    if (deletionForm === null) {
      return;
    }

    deletionForm.addEventListener('submit', () => {
      const keys = getDeletionKeys();
      delete keys[pasteId];
      setDeletionKeys(keys);
    });
  })();

  // expire old deletion keys
  (function() {
    const keys = getDeletionKeys();

    for (const key of Object.entries(keys)) {
      if (new Date >= new Date(key[1].expires)) {
        delete keys[key[0]];
      }
    }

    setDeletionKeys(keys);
  })();

  document
    .querySelectorAll('.paste-rendered-content pre[lang]')
    .forEach(pre => pre.classList.add(`language-${pre.lang}`));
})();
