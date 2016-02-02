class InputEvent {
    shiftDown: boolean;
    ctrlDown: boolean;
    altDown: boolean;
}

enum MouseButton {
    Left,
    Middle,
    Right,
}

class MousePress extends InputEvent {
    x: number;
    y: number;
    btn: MouseButton;
    down: boolean;
}

class MouseMove extends InputEvent {
    x: number;
    y: number;
}

class KeyPress extends InputEvent {
    key: number;
    down: boolean;
}

function interact(parent: HTMLElement, handler: (input: InputEvent) => void) {
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

    window.addEventListener("mousemove", function (e) {
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

function pauseEvent(e: Event) {
    if (e.stopPropagation) e.stopPropagation();
    if (e.preventDefault) e.preventDefault();
    e.cancelBubble = true;
    return false;
}