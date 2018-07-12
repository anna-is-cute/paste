(function() {
  if (localStorage.getItem('style') === 'dark') {
    document.getElementById('submit_button').setAttribute('data-theme', 'dark');
  }
})();

// eslint-disable-next-line no-unused-vars
function submitRegistration() {
  document.getElementById('registration_form').submit();
}
