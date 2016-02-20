var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var BasicMissile = (function (_super) {
    __extends(BasicMissile, _super);
    function BasicMissile(c, frame) {
        _super.call(this, c, frame);
    }
    BasicMissile.prototype.render = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "basic", 0, f, x, y);
    };
    return BasicMissile;
})(Missile);
//# sourceMappingURL=basic.js.map