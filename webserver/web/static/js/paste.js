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

  const tabs_container = document.getElementById('tabs-container');

  if (tabs_container !== null) {
    const rendered = document.getElementById('rendered-tab');
    const rendered_a = rendered.firstChild;

    const source = document.getElementById('source-tab');
    const source_a = source.firstChild;

    const rendered_content = tabs_container.querySelector('div#rendered-content');
    const source_content = tabs_container.querySelector('div#source-content');

    function swap(current, current_content, next, next_content) {
      current.classList.remove('is-active');
      next.classList.add('is-active');

      current_content.classList.add('is-not-displayed');
      next_content.classList.remove('is-not-displayed');
    }

    rendered_a.onclick = function() {
      swap(source, source_content, rendered, rendered_content);
    };
    source_a.onclick = function() {
      swap(rendered, rendered_content, source, source_content);
    };
  }
})();
