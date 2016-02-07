$(document).foundation();

$(document).ready(function() {
  $.ajax({
    url: "https://api.github.com/repos/tdfischer/pto/releases/latest",
    success: function(data) {
      console.log(data);
      $('#version').html(data.name);
    }
  });
});
