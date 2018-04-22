if (localStorage.getItem('dark') === null) {
  document.getElementById('style-dark').rel = 'stylesheet alternate';
} else {
  document.getElementById('style-light').rel = 'stylesheet alternate';
}
