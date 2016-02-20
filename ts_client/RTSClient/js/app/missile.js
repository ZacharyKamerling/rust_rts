var Missile = (function () {
    function Missile(c, frame, exploding) {
        this.frame_created = frame;
        this.exploding = exploding;
        this.misl_ID = c.getU16();
        this.x = c.getU16() / 2;
        this.y = c.getU16() / 2;
    }
    Missile.prototype.render = function (game, ctx, old, time, f, x, y) {
        throw new Error('Missile: render() is abstract');
    };
    Missile.prototype.speed = function () {
        throw new Error('Missile: speed() is abstract');
    };
    Missile.decodeMissile = function (data, frame, exploding) {
        var mislType = data.getU8();
        switch (mislType) {
            case 0:
                return new BasicMissile(data, frame, exploding);
            default:
                console.log("No missile of type " + mislType + " exists.");
                return null;
        }
    };
    return Missile;
})();
//# sourceMappingURL=missile.js.map