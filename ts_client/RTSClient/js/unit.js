define(["require", "exports"], function (require, exports) {
    var Unit = (function () {
        function Unit(c, time) {
            this.unitID = c.getF64();
            this.animID = c.getU8();
            this.x = c.getU16() / 64;
            this.y = c.getU16() / 64;
            this.facing = c.getU8() / 255 * 2 * Math.PI;
            this.timeCreated = time;
            this.unitID = c.getU16();
            this.x = c.getU16() / 64;
            this.y = c.getU16() / 64;
            this.animID = c.getU8();
            this.team = c.getU8();
            this.facing = c.getU8() * 2 * Math.PI / 255;
            this.health = c.getU8() / 255;
            this.progress = c.getU8() / 255;
        }
        Unit.prototype.coeff = function (old, time) {
            var diff = this.timeCreated - old.timeCreated;
            if (diff <= 0) {
                return 0;
            }
            else {
                return (time - this.timeCreated) / diff;
            }
        };
        Unit.prototype.render = function (game, old, time) {
            throw new Error('render() is abstract');
            // (facing + pi) % (2*pi) > otherFacing = turn counter clockwise
        };
        return Unit;
    })();
    exports.Unit = Unit;
});
//# sourceMappingURL=unit.js.map