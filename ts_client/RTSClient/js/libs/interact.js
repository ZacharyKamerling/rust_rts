var MouseButton;
(function (MouseButton) {
    MouseButton[MouseButton["Left"] = 0] = "Left";
    MouseButton[MouseButton["Middle"] = 1] = "Middle";
    MouseButton[MouseButton["Right"] = 2] = "Right";
})(MouseButton || (MouseButton = {}));
var MousePress = (function () {
    function MousePress() {
    }
    return MousePress;
})();
var MouseMove = (function () {
    function MouseMove() {
    }
    return MouseMove;
})();
function interact(parent, handler) {
    parent.draggable = false;
    document.addEventListener('contextmenu', function (e) {
        e.preventDefault();
    }, false);
    parent.addEventListener("mousedown", function (e) {
        var input = new MousePress();
        input.x = e.x;
        input.y = e.y;
        input.down = true;
        switch (e.button) {
            case 0:
                input.btn = MouseButton.Left;
                break;
            case 1:
                input.btn = MouseButton.Middle;
                break;
            case 2:
                input.btn = MouseButton.Right;
                break;
            default:
                break;
        }
        handler(input);
        pauseEvent(e);
    });
    window.addEventListener("mouseup", function (e) {
        var input = new MousePress();
        input.x = e.x;
        input.y = e.y;
        input.down = false;
        switch (e.button) {
            case 0:
                input.btn = MouseButton.Left;
                break;
            case 1:
                input.btn = MouseButton.Middle;
                break;
            case 2:
                input.btn = MouseButton.Right;
                break;
            default:
                break;
        }
        handler(input);
        pauseEvent(e);
    });
    window.addEventListener("mousemove", function (e) {
        var input = new MouseMove();
        input.x = e.x;
        input.y = e.y;
        handler(input);
        pauseEvent(e);
    });
    /*
    parent.addEventListener("touchstart", function (e: TouchEvent) {
        that.addTouches(e.touches);
        handler(that);
        pauseEvent(e);
    });

    parent.addEventListener("touchend", function (e: TouchEvent) {
        that.addTouches(e.touches);
        handler(that);
        pauseEvent(e);
    });

    parent.addEventListener("touchcancel", function (e: TouchEvent) {
        that.addTouches(e.touches);
        handler(that);
        pauseEvent(e);
    });

    parent.addEventListener("touchleave", function (e: TouchEvent) {
        that.addTouches(e.touches);
        handler(that);
        pauseEvent(e);
    });

    parent.addEventListener("touchmove", function (e: TouchEvent) {
        that.addTouches(e.touches);
        handler(that);
        pauseEvent(e);
    });
        
    parent.addEventListener("keydown", function (e) {
        that.keys[e.keyCode] = true;
        handler(that);
        pauseEvent(e);
    });

    parent.addEventListener("keyup", function (e) {
        that.keys[e.keyCode] = false;
        handler(that);
        pauseEvent(e);
    });
    */
}
function pauseEvent(e) {
    if (e.stopPropagation)
        e.stopPropagation();
    if (e.preventDefault)
        e.preventDefault();
    e.cancelBubble = true;
    return false;
}
//# sourceMappingURL=interact.js.map