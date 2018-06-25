(function() {
  if (localStorage.getItem('style') === 'dark') {
    document.getElementById('submit_button').setAttribute('data-theme', 'dark');
  }
})();

function submitRegistration() {
  document.getElementById('registration_form').submit();
}
