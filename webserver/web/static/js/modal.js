"use strict";

function _toConsumableArray(arr) { return _arrayWithoutHoles(arr) || _iterableToArray(arr) || _nonIterableSpread(); }

function _nonIterableSpread() { throw new TypeError("Invalid attempt to spread non-iterable instance"); }

function _iterableToArray(iter) { if (Symbol.iterator in Object(iter) || Object.prototype.toString.call(iter) === "[object Arguments]") return Array.from(iter); }

function _arrayWithoutHoles(arr) { if (Array.isArray(arr)) { for (var i = 0, arr2 = new Array(arr.length); i < arr.length; i++) { arr2[i] = arr[i]; } return arr2; } }

(function () {
  function openModal(e) {
    document.getElementById(e.dataset.modalId).classList.add('is-active');
  }

  function closeModal(e) {
    document.getElementById(e.dataset.modalId).classList.remove('is-active');
  }

  _toConsumableArray(document.getElementsByClassName('opens-modal')).forEach(function (e) {
    e.addEventListener('click', function () {
      return openModal(e);
    });

    if (e.tagName.toLowerCase() === 'button') {
      e.setAttribute('type', 'button');
    }
  });

  _toConsumableArray(document.getElementsByClassName('closes-modal')).forEach(function (e) {
    e.addEventListener('click', function () {
      return closeModal(e);
    });

    if (e.tagName.toLowerCase() === 'button') {
      e.setAttribute('type', 'button');
    }
  });
})();
//# sourceMappingURL=modal.js.map