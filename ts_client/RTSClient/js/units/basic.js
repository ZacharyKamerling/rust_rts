var __extends = (this && this.__extends) || function (d, b) {
    for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p];
    function __() { this.constructor = d; }
    d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
};
var Basic = (function (_super) {
    __extends(Basic, _super);
    function Basic(c, frame) {
        _super.call(this, c, frame);
    }
    Basic.prototype.getSightRadius = function () {
        return 12;
    };
    Basic.prototype.getRadius = function () {
        return 8;
    };
    Basic.prototype.render = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "basic", 0, f, x, y);
    };
    return Basic;
})(Unit);
//# sourceMappingURL=basic.js.map