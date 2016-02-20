var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var BasicUnit = (function (_super) {
    __extends(BasicUnit, _super);
    function BasicUnit(c, frame) {
        _super.call(this, c, frame);
        this.wpn_facing = c.getU8() * 2 * Math.PI / 255;
        this.wpn_anim = c.getU8();
    }
    BasicUnit.prototype.getSightRadius = function () {
        return 12;
    };
    BasicUnit.prototype.getRadius = function () {
        return 8;
    };
    BasicUnit.prototype.render = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "b_unit", 0, f, x, y);
        var wpn_f = Misc.turnTowards(old.wpn_facing, this.wpn_facing, Misc.angularDistance(old.wpn_facing, this.wpn_facing) * time);
        game.imageer.drawCentered(ctx, "b_wpn", 0, wpn_f, x, y);
    };
    return BasicUnit;
})(Unit);
//# sourceMappingURL=basic.js.map