var Fullscreen;
(function (Fullscreen) {
    function button(elem) {
        var btn = document.createElement("BUTTON");
        btn.textContent = 'Fullscreen';
        btn.style.position = 'absolute';
        btn.style.top = '0';
        btn.style.left = '0';
        btn.style.visibility = 'visible';
        btn.onclick = function (e) {
            if (elem.requestFullscreen) {
                elem.requestFullscreen();
            }
            else if (elem.webkitRequestFullscreen) {
                elem.webkitRequestFullscreen();
            }
        };
        document.addEventListener("webkitfullscreenchange", function (e) {
            if (btn.style.visibility === 'visible') {
                btn.style.visibility = 'hidden';
            }
            else {
                btn.style.visibility = 'visible';
            }
        });
        document.addEventListener("mozfullscreenchange", function (e) {
            if (btn.style.visibility === 'visible') {
                btn.style.visibility = 'hidden';
            }
            else {
                btn.style.visibility = 'visible';
            }
        });
        document.addEventListener("fullscreenchange", function (e) {
            if (btn.style.visibility === 'visible') {
                btn.style.visibility = 'hidden';
            }
            else {
                btn.style.visibility = 'visible';
            }
        });
        return btn;
    }
})(Fullscreen || (Fullscreen = {}));
//# sourceMappingURL=fullscreen_button.js.map