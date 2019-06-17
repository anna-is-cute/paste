/* global zxcvbn:false */

(function() {
  function checkMatch(pw, verify) {
    if (pw.value === verify.value && pw.value.length !== 0) {
      verify.classList.add('is-success');
    } else {
      verify.classList.remove('is-success');
    }
  }

  (function() {
    const verify = document.getElementById('password_verify');
    if (verify === null) {
      return;
    }
    verify.addEventListener('input', e => checkMatch(document.getElementById('password'), e.target));
  })();

  function doHides(pw, strength) {
    if (pw.value.length === 0) {
      strength.classList.add('is-not-displayed');
    } else {
      strength.classList.remove('is-not-displayed');
    }
  }

  (function() {
    const reveals = document.getElementsByName('password_reveal');
    for (const reveal of reveals) {
      reveal.addEventListener('click', () => {
        const pwField = reveal.parentElement.previousElementSibling.firstElementChild;
        pwField.type = pwField.type === 'password' ? 'text' : 'password';

        const icon = reveal.querySelector('i.fas');
        if (pwField.type === 'password') {
          icon.classList.remove('fa-eye-slash');
          icon.classList.add('fa-eye');
        } else {
          icon.classList.remove('fa-eye');
          icon.classList.add('fa-eye-slash');
        }
      });
    }
  })();

  function passwordStrength(pw) {
    checkMatch(pw, document.getElementById('password_verify'));

    const values = [];
    {
      const name = document.getElementById('name');
      if (name) {
        values.push(name.value);
      }
      const username = document.getElementById('username');
      if (username) {
        values.push(username.value);
      }
      const email = document.getElementById('email');
      if (email) {
        values.push(email.value);
      }
    }

    const password = pw.value;
    const strength = document.getElementById('strength');
    const progress = document.getElementById('strength_progress');
    const warning = document.getElementById('strength_warning');

    if (pw.getAttribute('data-bar') === 'hidden') {
      doHides(pw, progress);
    }

    if (password.length === 0) {
      strength.innerHTML = '';
      warning.innerHTML = '';
      progress.classList.add('is-danger');
      progress.classList.remove('is-warning');
      progress.classList.remove('is-success');
      return;
    }

    const z = zxcvbn(password, values);

    let message = `Time to crack your password: ${z.crack_times_display.offline_slow_hashing_1e4_per_second}`;
    message += ' <small><span class="has-text-grey-light tooltip is-tooltip-multiline is-dotted-underlined" data-tooltip="This is the time it would take an attacker to successfully guess your password. Increase your password complexity until you\'re comfortable with the amount of time.">What is this?</span></small>';
    strength.innerHTML = message;

    warning.innerHTML = `<br/>${z.feedback.warning}`;

    let color;
    switch (z.score) {
      case 0:
        color = 'is-danger';
        break;
      case 1:
        color = 'is-danger';
        break;
      case 2:
        color = 'is-warning';
        break;
      case 3:
        color = 'is-warning';
        break;
      case 4:
        color = 'is-success';
        break;
    }

    progress.classList.remove('is-danger');
    progress.classList.remove('is-warning');
    progress.classList.remove('is-success');
    progress.classList.add(color);
  }

  (function() {
    const pass = document.getElementById('password');
    if (pass !== null) {
      pass.addEventListener('input', e => passwordStrength(e.target));
    }
  })();
})();
