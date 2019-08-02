"use strict";

function _toConsumableArray(arr) { return _arrayWithoutHoles(arr) || _iterableToArray(arr) || _nonIterableSpread(); }

function _nonIterableSpread() { throw new TypeError("Invalid attempt to spread non-iterable instance"); }

function _iterableToArray(iter) { if (Symbol.iterator in Object(iter) || Object.prototype.toString.call(iter) === "[object Arguments]") return Array.from(iter); }

function _arrayWithoutHoles(arr) { if (Array.isArray(arr)) { for (var i = 0, arr2 = new Array(arr.length); i < arr.length; i++) { arr2[i] = arr[i]; } return arr2; } }

/* global CodeSass:false, luxon:false */
var pasteNum = 0;
var pasteEditors = {};

(function () {
  var DateTime = luxon.DateTime;
  var ws;
  connectWebSocket();

  function connectWebSocket() {
    ws = new WebSocket("wss://".concat(document.location.host, "/highlight/"));
    ws.addEventListener('close', connectWebSocket);
    ws.addEventListener('error', connectWebSocket);
    ws.addEventListener('message', function (msg) {
      // find the first newline and split the data on it
      var split = msg.data.indexOf('\n'); // the left side will be the paste number

      var id = Number(msg.data.substring(0, split)); // the right side will be the highlighted content

      var hl = msg.data.substring(split + 1); // get the code element of the given editor

      var code = pasteEditors[id].elCode; // add the lang attribute to the <pre> if necessary

      if (!code.parentElement.hasAttribute('lang')) {
        code.parentElement.setAttribute('lang', '');
      } // update the inner html of the <code> element


      code.innerHTML = hl;
    });
  }
  /**
   * Formats a UTC offset into "+04:30" format from a decimal like 4.5.
   *
   * @param {Number} i Decimal representing UTC offset
   * @returns {String} Formatted String
   */


  function prettyOffset(i) {
    // check if the offset is negative for formatting later. all the math will be done as if it were
    // positive
    var isNeg = i < 0;
    var input = Math.abs(i); // get the hour component by stripping off the fraction

    var hour = Math.floor(input); // subtract the hour component to get the fraction

    var frac = input - hour;
    var mins = 60 * frac; // pad with leading zeroes

    var hs = hour.toString().padStart(2, '0');
    var ms = mins.toString().padStart(2, '0');
    var pre = isNeg ? '-' : '+';
    return "".concat(pre).concat(hs, ":").concat(ms);
  }
  /**
   * @param {boolean} makeDate Whether to turn the ISO String into a Date
   * @returns {null | DateTime | String} The absolute expiry date set by the user, if set, otherwise
   * null. Returns an ISO string if makeDate is false, a DateTime if true.
   */


  function getAbsoluteExpiry(makeDate) {
    var date = document.getElementById('absolute-date');
    var time = document.getElementById('absolute-time');
    var tz = document.getElementById('absolute-timezone');

    if (date === null || time === null || tz === null) {
      return null;
    }

    var dateValue = date.value;
    var timeValue = time.value;
    var tzValue = tz.value;

    if (!dateValue || !timeValue || !tzValue) {
      return null;
    }

    var tzNum = Number(tzValue);
    var prettyTz = tzNum === 0 ? 'Z' : prettyOffset(tzNum);
    var dateString = "".concat(dateValue, "T").concat(timeValue, ":00.000").concat(prettyTz);
    var finalDate = DateTime.fromISO(dateString);

    if (makeDate) {
      return finalDate;
    }

    return finalDate.toString();
  }

  function getRelativeExpiry(makeDate) {
    var yearsElem = document.getElementById('relative-years');
    var daysElem = document.getElementById('relative-days');
    var hoursElem = document.getElementById('relative-hours');
    var minutesElem = document.getElementById('relative-minutes');

    if (yearsElem === null || daysElem === null || hoursElem === null || minutesElem === null) {
      return null;
    }

    var years = Number(yearsElem.value ? yearsElem.value : '0');
    var days = Number(daysElem.value ? daysElem.value : '0');
    var hours = Number(hoursElem.value ? hoursElem.value : '0');
    var minutes = Number(minutesElem.value ? minutesElem.value : '0');

    if (isNaN(years) || isNaN(days) || isNaN(hours) || isNaN(minutes)) {
      return null;
    }

    if (years + days + hours + minutes === 0) {
      return null;
    }

    var date = DateTime.local().plus({
      years: years,
      days: days,
      hours: hours,
      minutes: minutes
    });

    if (makeDate) {
      return date;
    }

    return date.toString();
  }
  /**
   * @returns { null | String } date
   */


  function getExpiry() {
    var expires = document.getElementById('expires');

    if (expires === null) {
      return null;
    }

    switch (expires.value) {
      case 'relative':
        return getRelativeExpiry(false);

      case 'absolute':
        return getAbsoluteExpiry(false);

      default:
        return null;
    }
  }

  function setTimezone(tz) {
    var tzSelect = document.getElementById('absolute-timezone');

    if (tzSelect === null) {
      return;
    }

    var offset = tz === undefined ? DateTime.local().offset / 60 : tz;

    _toConsumableArray(tzSelect.children).forEach(function (e) {
      if (Number(e.value) === offset) {
        e.setAttribute('selected', '');
      } else {
        e.removeAttribute('selected');
      }
    });
  }
  /**
   * Create the upload array for handling multiple files.
   *
   * @returns {[{name: string, language: string, content: string}]} Array of upload files.
   */


  function createUpload() {
    function getLanguage(parent) {
      var lang = parent.querySelector('select[name=file_language]').value;

      if (lang === '') {
        return null;
      }

      return lang;
    }

    var files = [];

    for (var _i = 0, _Object$values = Object.values(pasteEditors); _i < _Object$values.length; _i++) {
      var editor = _Object$values[_i];
      var parent = editor.editorRoot.parentElement.parentElement.parentElement;
      var file = {
        'name': parent.querySelector('input[name=file_name]').value,
        'language': getLanguage(parent),
        'content': editor.getCode()
      };
      var id = editor.editorRoot.parentElement.parentElement.parentElement.querySelector('input[name=id]');

      if (id !== null) {
        file.id = id.value;
      }

      files.push(file);
    }

    return files;
  }

  function codeFlaskSucksHighlight(editor) {
    // only use the websocket if it's connected
    if (ws.readyState === ws.OPEN) {
      // send a request over the websocket to highlight the code
      // FIXME: send the file name or the lang name correctly, not just rust
      ws.send("".concat(editor.pasteNum, "\nrust\nsnippet\n").concat(editor.elCode.innerText)); // when the response comes in, the editor will be updated. nothing else needs to be done here.
    }
  }
  /**
   * Create an editor.
   *
   * @param {HTMLElement} parent The file container.
   * @param {HTMLElement} el The element to convert into an editor.
   */


  function setUpEditor(parent, el) {
    var div = document.createElement('div');
    div.style.height = '400px';
    var editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'plaintext'
    });
    editor.pasteNum = pasteNum;
    var hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.editorRoot.insertAdjacentElement('afterend', hidden);
    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';
    editor.setHighlightCallback(codeFlaskSucksHighlight);
    var nameInput = parent.querySelector('input[name=file_name]');
    var langInput = parent.querySelector('select[name=file_language]');

    function updateLanguage() {
      function getSuffixFromName(name) {
        if (name === 'CMakeLists.txt') {
          return 'CMake';
        }

        return name.split('.').pop();
      }

      var suffix;

      if (langInput.value !== '') {
        suffix = langInput.value;
      } else if (nameInput.value !== '') {
        suffix = getSuffixFromName(nameInput.value);
      } // const lang = hljs.getLanguage(suffix) !== undefined ? suffix : 'plaintext';
      // editor.updateLanguage(lang);
      // editor.updateCode(editor.code);

    }

    nameInput.addEventListener('input', updateLanguage);
    langInput.addEventListener('change', updateLanguage);
    updateLanguage();
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    var toDelete = pasteNum;
    parent.querySelector('button[name=delete_button]').addEventListener('click', function () {
      return removeFile(toDelete);
    });
    pasteEditors[pasteNum] = editor;
    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  function addFile() {
    // get the base file for cloning (should be invisible if JS is running)
    var base = document.getElementById('base_file'); // deep clone the base

    var clone = base.cloneNode(true); // show the editor by removing the requires-no-js class that was on the base

    clone.classList.remove('requires-no-js');
    pasteNum += 1;
    clone.id = "file".concat(pasteNum); // set up an editor for each textarea in the base (should only be one)

    var _iteratorNormalCompletion = true;
    var _didIteratorError = false;
    var _iteratorError = undefined;

    try {
      for (var _iterator = clone.getElementsByTagName('textarea')[Symbol.iterator](), _step; !(_iteratorNormalCompletion = (_step = _iterator.next()).done); _iteratorNormalCompletion = true) {
        var ta = _step.value;
        setUpEditor(clone, ta);
      } // add the editor to the dom

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

    document.getElementById('end_of_files').insertAdjacentElement('beforebegin', clone);
    updateButtons();
  }
  /**
   * Remove a file. Will never remove the last file.
   *
   * @param {Number} num The number of the file to remove.
   */


  function removeFile(num) {
    if (Object.keys(pasteEditors).length === 1) {
      return;
    }

    var file = document.getElementById("file".concat(num));

    if (file === null) {
      return;
    }

    delete pasteEditors[num];
    file.remove();
    updateButtons();
  }

  function updateButtons() {
    var enabled = Object.keys(pasteEditors).length > 1;
    var _iteratorNormalCompletion2 = true;
    var _didIteratorError2 = false;
    var _iteratorError2 = undefined;

    try {
      for (var _iterator2 = document.getElementsByName('delete_button')[Symbol.iterator](), _step2; !(_iteratorNormalCompletion2 = (_step2 = _iterator2.next()).done); _iteratorNormalCompletion2 = true) {
        var button = _step2.value;

        if (enabled) {
          button.disabled = false;
        } else {
          button.disabled = true;
        }
      }
    } catch (err) {
      _didIteratorError2 = true;
      _iteratorError2 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion2 && _iterator2["return"] != null) {
          _iterator2["return"]();
        }
      } finally {
        if (_didIteratorError2) {
          throw _iteratorError2;
        }
      }
    }
  }

  function createEditors() {
    var _iteratorNormalCompletion3 = true;
    var _didIteratorError3 = false;
    var _iteratorError3 = undefined;

    try {
      for (var _iterator3 = document.querySelectorAll('textarea.editor')[Symbol.iterator](), _step3; !(_iteratorNormalCompletion3 = (_step3 = _iterator3.next()).done); _iteratorNormalCompletion3 = true) {
        var editor = _step3.value;
        pasteNum += 1;
        setUpEditor(editor.parentElement.parentElement.parentElement, editor);
      }
    } catch (err) {
      _didIteratorError3 = true;
      _iteratorError3 = err;
    } finally {
      try {
        if (!_iteratorNormalCompletion3 && _iterator3["return"] != null) {
          _iterator3["return"]();
        }
      } finally {
        if (_didIteratorError3) {
          throw _iteratorError3;
        }
      }
    }

    updateButtons();
  }

  document.getElementById('add_file').addEventListener('click', addFile);
  document.getElementById('paste_upload').addEventListener('submit', function (e) {
    var input = document.createElement('input');
    input.type = 'hidden';
    input.value = JSON.stringify(createUpload());
    input.name = 'upload_json';
    e.target.appendChild(input);
    var expiry = getExpiry();

    if (expiry !== null) {
      var expiresInput = document.createElement('input');
      expiresInput.type = 'hidden';
      expiresInput.value = expiry;
      expiresInput.name = 'expires';
      e.target.appendChild(expiresInput);
    }
  }); // create any initial editors

  createEditors(); // add an initial file if necessary

  if (Object.keys(pasteEditors).length === 0) {
    addFile();
  } // set the default timezone


  setTimezone();

  (function () {
    function inputsRequired(el, req) {
      el.querySelectorAll('input, select').forEach(function (e) {
        if (req) {
          e.setAttribute('required', '');
          return;
        }

        e.removeAttribute('required');
      });
    }

    var expires = document.getElementById('expires');

    if (expires === null) {
      return;
    }

    expires.addEventListener('change', function (e) {
      var expiry = e.target.value;
      var abs = document.getElementById('absolute-expiry');
      var rel = document.getElementById('relative-expiry');

      if (expiry === 'relative') {
        abs.classList.add('is-hidden');
        rel.classList.remove('is-hidden');
        inputsRequired(abs, false);
        inputsRequired(rel, true);
      } else if (expiry === 'absolute') {
        abs.classList.remove('is-hidden');
        rel.classList.add('is-hidden');
        inputsRequired(abs, true);
        inputsRequired(rel, false);
      } else {
        abs.classList.add('is-hidden');
        rel.classList.add('is-hidden');
        inputsRequired(abs, false);
        inputsRequired(rel, false);
      }
    });
  })();

  (function () {
    var abs = document.getElementById('absolute-expiry');
    var expirationDate = abs.dataset.expirationDate;

    if (expirationDate === undefined) {
      return;
    }

    var date = DateTime.fromISO(expirationDate);
    var year = date.year;
    var month = date.month.toString().padStart(2, '0');
    var day = date.day.toString().padStart(2, '0');
    document.getElementById('absolute-date').value = "".concat(year, "-").concat(month, "-").concat(day);
    var hour = date.hour.toString().padStart(2, '0');
    var minute = date.minute.toString().padStart(2, '0');
    document.getElementById('absolute-time').value = "".concat(hour, ":").concat(minute);
    document.getElementById('absolute-timezone').value = "".concat(date.offset / 60);
  })();
})();
//# sourceMappingURL=editor.js.map