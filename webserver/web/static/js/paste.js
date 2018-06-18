(function() {
  function openModal() {
    document.getElementById('deletion_modal').classList.add('is-active');
  }

  function closeModal() {
    document.getElementById('deletion_modal').classList.remove('is-active');
  }

  for (var e of document.getElementsByClassName('opens-modal')) {
    e.addEventListener('click', openModal);
  }

  for (var e of document.getElementsByClassName('closes-modal')) {
    e.addEventListener('click', closeModal);
  }

  function swap(current, current_content, next, next_content) {
    current.classList.remove('is-active');
    next.classList.add('is-active');

    current_content.classList.add('is-not-displayed');
    next_content.classList.remove('is-not-displayed');
  }

  for (const tabs_container of document.getElementsByClassName('paste-tabs-container')) {
    const file_id = tabs_container.dataset.id;
    const tab_links = document.getElementById(file_id + '-tab-links');

    const rendered = tab_links.querySelector('.paste-rendered-tab');
    const rendered_a = rendered.firstChild;

    const source = tab_links.querySelector('.paste-source-tab');
    const source_a = source.firstChild;

    const rendered_content = tabs_container.querySelector('div.paste-rendered-content');
    const source_content = tabs_container.querySelector('div.paste-source-content');

    rendered_a.addEventListener('click', function() {
      swap(source, source_content, rendered, rendered_content);
    });
    source_a.addEventListener('click', function() {
      swap(rendered, rendered_content, source, source_content);
    });
  }

  function getDeletionKeys() {
    var keys = localStorage.getItem('deletion_keys');

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
    const dk_elem = document.getElementById('deletion_key');

    if (dk_elem === null) {
      return;
    }

    const deletion_key = dk_elem.innerText;

    const keys = getDeletionKeys();

    const paste_id = dk_elem.dataset.pasteId;

    keys[paste_id] = {
      deletion_key: deletion_key,
      expires: new Date((new Date).getTime() + (30 * 24 * 60 * 60 * 1000)),
    };

    setDeletionKeys(keys);
  })();

  // check if we have a deletion key for this paste and insert it
  (function() {
    const dk_input = document.getElementById('deletion_key_input');

    if (dk_input === null) {
      return;
    }

    const paste_id = dk_input.dataset.pasteId;

    const keys = getDeletionKeys();

    const key = keys[paste_id];

    if (key === undefined) {
      return;
    }

    dk_input.value = key['deletion_key'];

    // add a listener for form submit to remove key from local storage
    const deletion_form = document.getElementById('deletion_form');

    if (deletion_form === null) {
      return;
    }

    deletion_form.addEventListener('submit', function() {
      const keys = getDeletionKeys();
      delete keys[paste_id];
      setDeletionKeys(keys);
    });
  })();

  // expire old deletion keys
  (function() {
    const keys = getDeletionKeys();

    for (const key of Object.entries(keys)) {
      if ((new Date) >= new Date(key[1]['expires'])) {
        delete keys[key[0]];
      }
    }

    setDeletionKeys(keys);
  })();
})();
