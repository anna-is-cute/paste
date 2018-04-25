(function() {
  if (localStorage.getItem('style') === 'dark') {
    document.getElementById('recaptcha').setAttribute('data-theme', 'dark');
  }
})();
