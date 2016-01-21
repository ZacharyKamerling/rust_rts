var Tilemap = (function () {
    function Tilemap(w, h, v) {
        var n = w * h;
        this.tiles = new Array(n);
        this.tw = w;
        this.th = h;
        while (n--) {
            this.tiles[n] = v;
        }
    }
    Tilemap.prototype.setTile = function (x, y, str) {
        if (x >= 0 && y >= 0 && x < this.tw && y < this.th) {
            this.tiles[y * this.tw + x] = str;
        }
    };
    Tilemap.prototype.getTile = function (x, y) {
        if (x >= 0 && y >= 0 && x < this.tw && y < this.th) {
            return this.tiles[y * this.tw + x];
        }
        else {
            return null;
        }
    };
    return Tilemap;
})();
//# sourceMappingURL=tilemap.js.map