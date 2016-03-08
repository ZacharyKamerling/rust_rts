var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var BasicUnit = (function (_super) {
    __extends(BasicUnit, _super);
    function BasicUnit(c, frame) {
        if (c) {
            _super.call(this, c, frame);
            this.wpn_facing = c.getU8() * 2 * Math.PI / 255;
            this.wpn_anim = c.getU8();
        }
    }
    BasicUnit.prototype.copycat = function (unit) {
        _super.prototype.copycat.call(this, unit);
        unit.wpn_anim = this.wpn_anim;
        unit.wpn_facing = this.wpn_facing;
    };
    BasicUnit.prototype.clone = function () {
        var u = new BasicUnit(null, 0);
        this.copycat(u);
        return u;
    };
    BasicUnit.prototype.getSightRadius = function () {
        return 12.0;
    };
    BasicUnit.prototype.getRadius = function () {
        return 0.6;
    };
    BasicUnit.prototype.step = function (time, oldUnit, newUnit) {
        _super.prototype.step.call(this, time, oldUnit, newUnit);
        var f1 = oldUnit.wpn_facing;
        var f2 = newUnit.wpn_facing;
        this.wpn_facing = Misc.turnTowards(this.wpn_facing, f2, Misc.angularDistance(f1, f2) * time);
    };
    BasicUnit.prototype.render = function (game, ctx, x, y) {
        game.imageer.drawCentered(ctx, "b_unit", 0, this.facing, x, y);
        game.imageer.drawCentered(ctx, "b_wpn", 0, this.wpn_facing, x, y);
    };
    return BasicUnit;
})(Unit);
//# sourceMappingURL=basic_unit.js.map