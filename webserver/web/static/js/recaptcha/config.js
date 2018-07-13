(function() {
  window['___grecaptcha_cfg'] = {
    'render': 'onload'
  };
  window['grecaptcha'] = {
    ready: function(f) {
      (window['___grecatpcha_cfg']['fns'] = window['___grecaptcha_cfg']['fns'] || []).push(f);
    },
  };
  window['__google_recaptcha_client'] = true
})();
