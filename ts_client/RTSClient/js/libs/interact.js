var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var InputEvent = (function () {
    function InputEvent() {
    }
    return InputEvent;
})();
var MouseButton;
(function (MouseButton) {
    MouseButton[MouseButton["Left"] = 0] = "Left";
    MouseButton[MouseButton["Middle"] = 1] = "Middle";
    MouseButton[MouseButton["Right"] = 2] = "Right";
})(MouseButton || (MouseButton = {}));
var MousePress = (function (_super) {
    __extends(MousePress, _super);
    function MousePress() {
        _super.apply(this, arguments);
    }
    return MousePress;
})(InputEvent);
var MouseMove = (function (_super) {
    __extends(MouseMove, _super);
    function MouseMove() {
        _super.apply(this, arguments);
    }
    return MouseMove;
})(InputEvent);
var KeyPress = (function (_super) {
    __extends(KeyPress, _super);
    function KeyPress() {
        _super.apply(this, arguments);
    }
    return KeyPress;
})(InputEvent);
function interact(parent, handler) {
    parent.draggable = false;
    document.addEventListener('contextmenu', function (e) {
        e.preventDefault();
    }, false);
    parent.addEventListener("mousedown", function (e) {
        var input = new MousePress();
        input.shiftDown = e.shiftKey;
        input.ctrlDown = e.ctrlKey;
        input.altDown = e.altKey;
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
        input.shiftDown = e.shiftKey;
        input.ctrlDown = e.ctrlKey;
        input.altDown = e.altKey;
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
    parent.addEventListener("mousemove", function (e) {
        var input = new MouseMove();
        input.shiftDown = e.shiftKey;
        input.ctrlDown = e.ctrlKey;
        input.altDown = e.altKey;
        input.x = e.x;
        input.y = e.y;
        handler(input);
        pauseEvent(e);
    });
    parent.addEventListener("keydown", function (e) {
        var input = new KeyPress();
        input.shiftDown = e.shiftKey;
        input.ctrlDown = e.ctrlKey;
        input.altDown = e.altKey;
        input.key = e.keyCode;
        input.down = true;
        handler(input);
        pauseEvent(e);
    });
    parent.addEventListener("keyup", function (e) {
        var input = new KeyPress();
        input.shiftDown = e.shiftKey;
        input.ctrlDown = e.ctrlKey;
        input.altDown = e.altKey;
        input.key = e.keyCode;
        input.down = false;
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