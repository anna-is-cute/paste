"use strict";

(function () {
  function swap(current, currentContent, next, nextContent) {
    current.classList.remove('is-active');
    next.classList.add('is-active');
    currentContent.classList.add('is-not-displayed');
    nextContent.classList.remove('is-not-displayed');
  }

  var _iteratorNormalCompletion = true;
  var _didIteratorError = false;
  var _iteratorError = undefined;

  try {
    var _loop = function _loop() {
      var tabsContainer = _step.value;
      var fileId = tabsContainer.dataset.id;
      var tabLinks = document.getElementById("".concat(fileId, "-tab-links"));
      var rendered = tabLinks.querySelector('.paste-rendered-tab');
      var renderedA = rendered.firstChild;
      var source = tabLinks.querySelector('.paste-source-tab');
      var sourceA = source.firstChild;
      var renderedContent = tabsContainer.querySelector('div.paste-rendered-content');
      var sourceContent = tabsContainer.querySelector('div.paste-source-content');
      renderedA.addEventListener('click', function () {
        return swap(source, sourceContent, rendered, renderedContent);
      });
      sourceA.addEventListener('click', function () {
        return swap(rendered, renderedContent, source, sourceContent);
      });
    };

    for (var _iterator = document.getElementsByClassName('paste-tabs-container')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
      _loop();
    }
  } catch (err) {
    _didIteratorError = true;
    _iteratorError = err;
  } finally {
    try {
      if (!_iteratorNormalCompletion && _iterator["return"] != null) {
        _iterator["return"]();
      }
    } finally {
      if (_didIteratorError) {
        throw _iteratorError;
      }
    }
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
  } // check if the page is displaying a deletion key and add it to local storage


  (function () {
    var dkElem = document.getElementById('deletion_key');

    if (dkElem === null) {
      return;
    }

    var deletionKey = dkElem.innerText;
    var keys = getDeletionKeys();
    var pasteId = dkElem.dataset.pasteId;
    keys[pasteId] = {
      deletionKey: deletionKey,
      expires: new Date(new Date().getTime() + 30 * 24 * 60 * 60 * 1000)
    };
    setDeletionKeys(keys);
  })(); // check if we have a deletion key for this paste and insert it


  (function () {
    var dkInput = document.getElementById('deletion_key_input');

    if (dkInput === null) {
      return;
    }

    var pasteId = dkInput.dataset.pasteId;
    var keys = getDeletionKeys();
    var key = keys[pasteId];

    if (key === undefined) {
      return;
    }

    dkInput.value = key.deletionKey; // add a listener for form submit to remove key from local storage

    var deletionForm = document.getElementById('deletion_form');

    if (deletionForm === null) {
      return;
    }

    deletionForm.addEventListener('submit', function () {
      var keys = getDeletionKeys();
      delete keys[pasteId];
      setDeletionKeys(keys);
    });
  })(); // expire old deletion keys


  (function () {
    var keys = getDeletionKeys();

    for (var _i = 0, _Object$entries = Object.entries(keys); _i < _Object$entries.length; _i++) {
      var key = _Object$entries[_i];

      if (new Date() >= new Date(key[1].expires)) {
        delete keys[key[0]];
      }
    }

    setDeletionKeys(keys);
  })();

  document.querySelectorAll('.paste-rendered-content pre[lang]').forEach(function (pre) {
    return pre.classList.add("language-".concat(pre.lang));
  });
})();
//# sourceMappingURL=paste.js.map