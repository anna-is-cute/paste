(function() {
  function checkMatch(pw, verify) {
    if (pw.value === verify.value && pw.value.length !== 0) {
      verify.classList.add('is-success');
    } else {
      verify.classList.remove('is-success');
    }
  }

  document.getElementById('password_verify').oninput = function() {
    checkMatch(document.getElementById('password'), this);
  }

  function doHides(pw, strength) {
    if (pw.value.length === 0) {
      strength.style = 'display: none;';
    } else {
      strength.style = null;
    }
  }

  function passwordStrength(pw) {
    checkMatch(pw, document.getElementById('password_verify'));

    var values = [
      document.getElementById('name').value,
      document.getElementById('username').value,
      document.getElementById('email').value,
    ];

    var password = pw.value;
    var strength = document.getElementById('strength');
    var progress = document.getElementById('strength_progress');
    var warning = document.getElementById('strength_warning');

    if (pw.getAttribute('data-bar') === 'hidden') {
      doHides(pw, strength_progress);
    }

    if (password.length === 0) {
      strength.innerHTML = '';
      warning.innerHTML = '';
      progress.classList.add('is-danger');
      progress.classList.remove('is-warning');
      progress.classList.remove('is-success');
      return;
    }

    var z = zxcvbn(password, values);

    var message = 'Time to crack your password: ' + z.crack_times_display.offline_slow_hashing_1e4_per_second;
    message += ' <small><span class="has-text-grey-light tooltip is-tooltip-multiline" style="text-decoration: dotted underline;" data-tooltip="This is the time it would take an attacker to successfully guess your password. Increase your password complexity until you\'re comfortable with the amount of time.">What is this?</i></small>';
    strength.innerHTML = message;

    warning.innerHTML = '<br/>' + z.feedback.warning;

    var color;
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

    var classes = progress.classList;
    progress.classList.remove(progress.className.split(' ').pop());
    progress.classList.add(color);
  }

  document.getElementById('password').oninput = function() {
    passwordStrength(this);
  };
})();
