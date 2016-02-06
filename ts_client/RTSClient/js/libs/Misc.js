var Misc;
(function (Misc) {
    function convertCanvasToImage(canvas) {
        var image = new Image();
        image.src = canvas.toDataURL("image/png");
        return image;
    }
    Misc.convertCanvasToImage = convertCanvasToImage;
    function normalizeAngle(f) {
        while (f > Math.PI * 2.0) {
            f -= Math.PI * 2.0;
        }
        while (f < 0.0) {
            f += Math.PI * 2.0;
        }
        return f;
    }
    Misc.normalizeAngle = normalizeAngle;
    function angularDistance(a, b) {
        var dists = Math.abs(a - b);
        if (dists > Math.PI) {
            return 2.0 * Math.PI - dists;
        }
        else {
            return dists;
        }
    }
    Misc.angularDistance = angularDistance;
    // Angle to turn, angle to turn towards, amount to turn
    function turnTowards(a, b, turn) {
        var dist = angularDistance(a, b);
        if (turn > dist) {
            return b;
        }
        else {
            if (a > b) {
                if (a - b > Math.PI) {
                    return normalizeAngle(a + turn);
                }
                else {
                    return normalizeAngle(a - turn);
                }
            }
            else {
                if (b - a > Math.PI) {
                    return normalizeAngle(a - turn);
                }
                else {
                    return normalizeAngle(a + turn);
                }
            }
        }
    }
    Misc.turnTowards = turnTowards;
})(Misc || (Misc = {}));
//# sourceMappingURL=misc.js.map