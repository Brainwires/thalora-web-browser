// External deferred script of js_app.html: handlers for the MCP end-to-end
// tests of page JavaScript (tests/protocols/e2e_fixture_tests.rs).
(function () {
  function byId(id) { return document.getElementById(id); }
  function token() { return document.querySelector('#js-form input[name=token]'); }

  byId('deferred').textContent = 'deferred script ran after ' + byId('boot').textContent;

  // A click handler that changes the DOM and logs
  var clicks = 0;
  byId('counter').addEventListener('click', function () {
    clicks += 1;
    byId('counter').textContent = 'Clicked ' + clicks + ' times';
    console.warn('counter clicked ' + clicks);
  });

  // Content that arrives from a timer, later than a click settles
  byId('load-more').addEventListener('click', function () {
    setTimeout(function () {
      var more = document.createElement('p');
      more.id = 'more';
      more.textContent = 'more content from a timer';
      byId('more-slot').appendChild(more);
    }, 700);
  });

  // A submit handler that fills a hidden field and lets the form go
  byId('js-form').addEventListener('submit', function () {
    token().value = 'set-by-submit-handler';
  });

  // A form submitted by script (no submit event, as in a browser)
  byId('submit-by-script').addEventListener('click', function () {
    token().value = 'set-by-click-handler';
    byId('js-form').submit();
  });

  // A navigation started by script
  byId('go-index').addEventListener('click', function () {
    location.href = '/index.html';
  });

  byId('keys').addEventListener('keydown', function (event) {
    byId('keylog').textContent += '[' + event.key + ']';
  });
  byId('hover-target').addEventListener('mouseover', function () {
    byId('hover-target').textContent = 'hovered';
  });
  window.addEventListener('scroll', function () {
    byId('scrolled').textContent = 'scrolled';
  });
})();
