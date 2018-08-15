'use strict';

function _toConsumableArray(arr) { if (Array.isArray(arr)) { for (var i = 0, arr2 = Array(arr.length); i < arr.length; i++) { arr2[i] = arr[i]; } return arr2; } else { return Array.from(arr); } }

(function () {
  function openModal(e) {
    document.getElementById(e.dataset.modalId).classList.add('is-active');
  }

  function closeModal(e) {
    document.getElementById(e.dataset.modalId).classList.remove('is-active');
  }

  [].concat(_toConsumableArray(document.getElementsByClassName('opens-modal'))).forEach(function (e) {
    return e.addEventListener('click', function () {
      return openModal(e);
    });
  });

  [].concat(_toConsumableArray(document.getElementsByClassName('closes-modal'))).forEach(function (e) {
    return e.addEventListener('click', function () {
      return closeModal(e);
    });
  });
})();
//# sourceMappingURL=modal.js.map