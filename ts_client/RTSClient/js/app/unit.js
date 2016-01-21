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
        throw new Error('getSightRadius() is abstract');
    };
    Unit.prototype.render = function (game, ctx, old, time, f, x, y) {
        game.imageer.drawCentered(ctx, "", 0, f, x, y);
    };
    Unit.prototype.renderFOW = function (game, ctx, old, time, f, x, y) {
        ctx.beginPath();
        ctx.fillStyle = '#000000';
        ctx.arc(x, y, Game.TILESIZE * this.getSightRadius(), 0, 2 * Math.PI, true);
        ctx.fill();
    };
    return Unit;
})();
//# sourceMappingURL=unit.js.map