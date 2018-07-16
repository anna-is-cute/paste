/* global hljs:false, CodeSass:false, luxon:false */

let pasteNum = 0;
const pasteEditors = {};

(function() {
  const DateTime = luxon.DateTime;

  /**
   * Formats a UTC offset into "+04:30" format from a decimal like 4.5.
   *
   * @param {Number} i Decimal representing UTC offset
   * @returns {String} Formatted String
   */
  function prettyOffset(i) {
    // check if the offset is negative for formatting later. all the math will be done as if it were
    // positive
    const isNeg = i < 0;

    const input = Math.abs(i);

    // get the hour component by stripping off the fraction
    const hour = Math.floor(input);

    // subtract the hour component to get the fraction
    const frac = input - hour;

    const mins = 60 * frac;

    // pad with leading zeroes
    const hs = hour.toString().padStart(2, '0');
    const ms = mins.toString().padStart(2, '0');

    const pre = isNeg ? '-' : '+';
    return `${pre}${hs}:${ms}`;
  }

  /**
   * @param {boolean} makeDate Whether to turn the ISO String into a Date
   * @returns {null | DateTime | String} The absolute expiry date set by the user, if set, otherwise
   * null. Returns an ISO string if makeDate is false, a DateTime if true.
   */
  function getAbsoluteExpiry(makeDate) {
    const date = document.getElementById('absolute-date');
    const time = document.getElementById('absolute-time');
    const tz = document.getElementById('absolute-timezone');

    if (date === null || time === null || tz === null) {
      return null;
    }

    const dateValue = date.value;
    const timeValue = time.value;
    const tzValue = tz.value;

    if (!dateValue || !timeValue || !tzValue) {
      return null;
    }

    const tzNum = Number(tzValue);

    const prettyTz = tzNum === 0 ? 'Z' : prettyOffset(tzNum);

    const dateString = `${dateValue}T${timeValue}:00.000${prettyTz}`;

    const finalDate = DateTime.fromISO(dateString);

    if (makeDate) {
      return finalDate;
    }

    return finalDate.toString();
  }

  function getRelativeExpiry(makeDate) {
    const yearsElem = document.getElementById('relative-years');
    const daysElem = document.getElementById('relative-days');
    const hoursElem = document.getElementById('relative-hours');
    const minutesElem = document.getElementById('relative-minutes');

    if (yearsElem === null || daysElem === null || hoursElem === null || minutesElem === null) {
      return null;
    }

    const years = Number(yearsElem.value ? yearsElem.value : '0');
    const days = Number(daysElem.value ? daysElem.value : '0');
    const hours = Number(hoursElem.value ? hoursElem.value : '0');
    const minutes = Number(minutesElem.value ? minutesElem.value : '0');

    if (isNaN(years) || isNaN(days) || isNaN(hours) || isNaN(minutes)) {
      return null;
    }

    if (years + days + hours + minutes === 0) {
      return null;
    }

    const date = DateTime.local().plus({
      years,
      days,
      hours,
      minutes,
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
    const expires = document.getElementById('expires');
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
    const tzSelect = document.getElementById('absolute-timezone');
    if (tzSelect === null) {
      return;
    }

    const offset = tz === undefined ? DateTime.local().offset / 60 : tz;
    [...tzSelect.children].forEach(e => {
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
      const lang = parent.querySelector('select[name=file_language]').value;
      if (lang === '') {
        return null;
      }
      return lang;
    }

    const files = [];
    for (const editor of Object.values(pasteEditors)) {
      const parent = editor.editorRoot.parentElement.parentElement.parentElement;
      const file = {
        'name': parent.querySelector('input[name=file_name]').value,
        'language': getLanguage(parent),
        'content': editor.getCode(),
      };
      const id = editor.editorRoot.parentElement.parentElement.parentElement.querySelector('input[name=id]');
      if (id !== null) {
        file.id = id.value;
      }
      files.push(file);
    }
    return files;
  }

  function codeFlaskSucksHighlight(editor) {
    hljs.highlightBlock(editor.elCode);
    // remove the extra classes hljs adds without asking
    for (const clazz of editor.elCode.classList) {
      if (clazz !== 'hljs' && clazz !== 'codeflask__code' && !clazz.startsWith('language-')) {
        editor.elCode.classList.remove(clazz);
      }
    }
  }

  /**
   * Create an editor.
   *
   * @param {HTMLElement} parent The file container.
   * @param {HTMLElement} el The element to convert into an editor.
   */
  function setUpEditor(parent, el) {
    const div = document.createElement('div');

    div.style.height = '400px';

    const editor = new CodeSass(div, {
      defaultTheme: false,
      lineNumbers: true,
      language: 'plaintext',
    });

    const hidden = document.createElement('input');
    hidden.type = 'hidden';
    hidden.name = 'file_content';
    hidden.id = 'hidden_content';
    editor.editorRoot.insertAdjacentElement('afterend', hidden);

    editor.elCode.style.background = 'none';
    editor.elCode.style.padding = '0';

    editor.setHighlightCallback(codeFlaskSucksHighlight);

    const nameInput = parent.querySelector('input[name=file_name]');
    const langInput = parent.querySelector('select[name=file_language]');

    function updateLanguage() {
      let suffix;
      if (langInput.value !== '') {
        suffix = langInput.value;
      } else if (nameInput.value !== '') {
        suffix = nameInput.value.split('.').pop();
      }
      const lang = hljs.getLanguage(suffix) !== undefined ? suffix : 'plaintext';
      editor.updateLanguage(lang);
      editor.updateCode(editor.code);
    }

    nameInput.addEventListener('input', updateLanguage);
    langInput.addEventListener('change', updateLanguage);

    updateLanguage();
    editor.updateCode(el.value);
    editor.createLineNumbers(); // TODO: fix this in codesass

    const toDelete = pasteNum;
    parent
      .querySelector('button[name=delete_button]')
      .addEventListener('click', () => removeFile(toDelete));

    pasteEditors[pasteNum] = editor;

    el.insertAdjacentElement('beforebegin', div);
    el.remove();
  }

  function addFile() {
    // get the base file for cloning (should be invisible if JS is running)
    const base = document.getElementById('base_file');

    // deep clone the base
    const clone = base.cloneNode(true);

    // show the editor by removing the requires-no-js class that was on the base
    clone.classList.remove('requires-no-js');

    pasteNum += 1;
    clone.id = `file${pasteNum}`;

    // set up an editor for each textarea in the base (should only be one)
    for (const ta of clone.getElementsByTagName('textarea')) {
      setUpEditor(clone, ta);
    }

    // add the editor to the dom
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

    const file = document.getElementById(`file${num}`);

    if (file === null) {
      return;
    }

    delete pasteEditors[num];

    file.remove();

    updateButtons();
  }

  function updateButtons() {
    const enabled = Object.keys(pasteEditors).length > 1;
    for (const button of document.getElementsByName('delete_button')) {
      if (enabled) {
        button.disabled = false;
      } else {
        button.disabled = true;
      }
    }
  }

  function createEditors() {
    for (const editor of document.querySelectorAll('textarea.editor')) {
      pasteNum += 1;
      setUpEditor(editor.parentElement.parentElement.parentElement, editor);
    }
    updateButtons();
  }

  document.getElementById('add_file').addEventListener('click', addFile);

  document.getElementById('paste_upload').addEventListener('submit', e => {
    const input = document.createElement('input');
    input.type = 'hidden';
    input.value = JSON.stringify(createUpload());
    input.name = 'upload_json';

    e.target.appendChild(input);

    const expiry = getExpiry();
    if (expiry !== null) {
      const expiresInput = document.createElement('input');
      expiresInput.type = 'hidden';
      expiresInput.value = expiry;
      expiresInput.name = 'expires';

      e.target.appendChild(expiresInput);
    }
  });

  // create any initial editors
  createEditors();

  // add an initial file if necessary
  if (Object.keys(pasteEditors).length === 0) {
    addFile();
  }

  // set the default timezone
  setTimezone();

  (function() {
    function inputsRequired(el, req) {
      el.querySelectorAll('input, select').forEach(e => {
        if (req) {
          e.setAttribute('required', '');
          return;
        }
        e.removeAttribute('required');
      });
    }

    const expires = document.getElementById('expires');
    if (expires === null) {
      return;
    }
    expires.addEventListener('change', e => {
      const expiry = e.target.value;
      const abs = document.getElementById('absolute-expiry');
      const rel = document.getElementById('relative-expiry');
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

  (function() {
    const abs = document.getElementById('absolute-expiry');
    const expirationDate = abs.dataset.expirationDate;
    if (expirationDate === undefined) {
      return;
    }

    const date = DateTime.fromISO(expirationDate);

    const year = date.year;
    const month = date.month.toString().padStart(2, '0');
    const day = date.day.toString().padStart(2, '0');
    document.getElementById('absolute-date').value = `${year}-${month}-${day}`;

    const hour = date.hour.toString().padStart(2, '0');
    const minute = date.minute.toString().padStart(2, '0');
    document.getElementById('absolute-time').value = `${hour}:${minute}`;
  })();
})();
