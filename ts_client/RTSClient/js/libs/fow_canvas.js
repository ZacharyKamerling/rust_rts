var FOWCanvas = (function () {
    function FOWCanvas(w, h) {
        this.sprites = {};
        this.fow_canvas = document.createElement("canvas");
        this.fow_canvas.width = w;
        this.fow_canvas.height = h;
    }
    FOWCanvas.prototype.setWidthAndHeight = function (w, h) {
        if (this.fow_canvas.width !== w || this.fow_canvas.height !== h) {
            this.fow_canvas.width = w;
            this.fow_canvas.height = h;
        }
    };
    FOWCanvas.prototype.beginRevealing = function () {
        var ctx = this.fow_canvas.getContext("2d");
        ctx.imageSmoothingEnabled = false;
        ctx.globalCompositeOperation = 'source-over';
        ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
        ctx.fillStyle = "rgba(0, 0, 0, 0.85)";
        ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
        ctx.globalCompositeOperation = 'destination-out';
    };
    FOWCanvas.prototype.revealArea = function (x, y, r) {
        var img = this.sprites[r];
        if (img) {
            var ctx = this.fow_canvas.getContext("2d");
            var w = ctx.canvas.width;
            var h = ctx.canvas.height;
            var dx = (x - r);
            var dy = (y - r);
            var dr = r * 2;
            ctx.drawImage(img, dx, dy);
        }
        else {
            // Create fog of war sprite
            var fows = document.createElement("canvas");
            fows.width = r * 2;
            fows.height = r * 2;
            var ctx = fows.getContext("2d");
            ctx.beginPath();
            ctx.fillStyle = '#000000';
            ctx.arc(r, r, r, 0, 2 * Math.PI, true);
            ctx.fill();
            this.sprites[r] = convertCanvasToImage(fows);
            // Use sprite
            ctx = this.fow_canvas.getContext("2d");
            var w = ctx.canvas.width;
            var h = ctx.canvas.height;
            var dx = (x - r);
            var dy = (y - r);
            var dr = r * 2;
            ctx.drawImage(this.sprites[r], dx, dy);
        }
    };
    FOWCanvas.prototype.paintFOW = function (ctx) {
        ctx.drawImage(this.fow_canvas, 0, 0, ctx.canvas.width, ctx.canvas.height);
    };
    return FOWCanvas;
})();
//# sourceMappingURL=fow_canvas.js.map