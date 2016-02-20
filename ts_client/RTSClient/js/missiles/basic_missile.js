var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var BasicMissile = (function (_super) {
    __extends(BasicMissile, _super);
    function BasicMissile(c, frame, exploding) {
        _super.call(this, c, frame, exploding);
    }
    BasicMissile.prototype.render = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "b_misl", 0, f, x, y);
    };
    BasicMissile.prototype.renderExplosion = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "b_misl", 0, f, x, y);
    };
    BasicMissile.prototype.speed = function () {
        return Game.TILESIZE * 10.0 / 10.0;
    };
    return BasicMissile;
})(Missile);
//# sourceMappingURL=basic_missile.js.map