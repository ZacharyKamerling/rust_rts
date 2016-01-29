var FOW;
(function (FOW) {
    var FOWPainter = (function () {
        function FOWPainter(w, h, r) {
            // Fog of war sprite
            var fows = document.createElement("canvas");
            fows.width = w;
            fows.height = h;
            var ctx = fows.getContext("2d");
            ctx.save();
            ctx.beginPath();
            ctx.fillStyle = '#000000';
            ctx.arc(r, r, r, 0, 2 * Math.PI, true);
            ctx.fill();
            ctx.restore();
            this.fow_sprite = convertCanvasToImage(fows);
        }
        FOWPainter.prototype.revealArea = function (ctx, x, y, r) {
            var w = ctx.canvas.width;
            var h = ctx.canvas.height;
            var dx = (x - r);
            var dy = (y - r);
            var dr = r * 2;
            ctx.drawImage(this.fow_sprite, dx, dy, dr, dr);
        };
        return FOWPainter;
    })();
    FOW.FOWPainter = FOWPainter;
    var FOWCanvas = (function () {
        function FOWCanvas(canvas) {
        }
        FOWCanvas.prototype.fowReveal = function (fowp) {
            fowp.revealArea(this.fow_canvas.getContext("2d"), 0, 0, 0);
        };
        return FOWCanvas;
    })();
    function paint() {
    }
})(FOW || (FOW = {}));
//# sourceMappingURL=fog_of_war.js.map