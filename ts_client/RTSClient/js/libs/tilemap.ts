class Tilemap<A> {
    private tiles: A[];
    private tw: number;
    private th: number;

    constructor(w: number, h: number, v: A) {
        var n = w * h;
        this.tiles = new Array(n);
        this.tw = w;
        this.th = h;

        while (n--) {
            this.tiles[n] = v;
        }
    }

    setTile(x: number, y: number, str: A) {
        if (x >= 0 && y >= 0 && x < this.tw && y < this.th) {
            this.tiles[y * this.tw + x] = str;
        }
    }

    getTile(x: number, y: number): A {
        if (x >= 0 && y >= 0 && x < this.tw && y < this.th) {
            return this.tiles[y * this.tw + x];
        }
        else {
            return null;
        }
    }
}