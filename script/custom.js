/**************************
 *        Off-canvas      *
 **************************/
$(document).ready(function () {
	$('[data-toggle="offcanvas"]').click(function () {
	$(this).toggleClass("active");
	$('.row-offcanvas').toggleClass('active')
	});
});


/**************************
 *      Scroll-to-Top     *
 **************************/
$(document).ready(function(){
  $('body').append('<div id="toTop" class="btn btn-info"><span class="fa fa-angle-up"></span></div>');
    $(window).scroll(function () {
        if ($(this).scrollTop() != 0) {
            $('#toTop').fadeIn();
        } else {
            $('#toTop').fadeOut();
        }
        })
}); 
$('#toTop').click(function(){
    $("html, body").animate({ scrollTop: 0 }, 600);
    return false;
});

/**************************
 *      Nav-Active     *
 **************************/
$(function() {
     var pgurl = window.location.href.substr(window.location.href
.lastIndexOf("/")+1);
     $("nav ul a").each(function(){
          if($(this).attr("href") == pgurl || $(this).attr("href") == '' )
          $(this).addClass("active");
     })
});
