var Unit = (function () {
    function Unit(c, frame) {
        this.frame_created = frame;
        this.unit_ID = c.getU16();
        this.x = c.getU16() / 2;
        this.y = c.getU16() / 2;
        this.anim_ID = c.getU8();
        this.team = c.getU8();
        this.facing = c.getU8() * 2 * Math.PI / 255;
        this.health = c.getU8() / 255;
        this.progress = c.getU8() / 255;
    }
    Unit.prototype.getSightRadius = function () {
        throw new Error('Unit: getSightRadius() is abstract');
    };
    Unit.prototype.getRadius = function () {
        throw new Error('Unit: getRadius() is abstract');
    };
    Unit.prototype.render = function (game, ctx, old, time, f, x, y) {
        throw new Error('Unit: render() is abstract');
    };
    Unit.decodeUnit = function (data, frame) {
        var unitType = data.getU8();
        switch (unitType) {
            case 0:
                return new BasicUnit(data, frame);
            default:
                console.log("No unit of type " + unitType + " exists.");
                return null;
        }
    };
    return Unit;
})();
//# sourceMappingURL=unit.js.map