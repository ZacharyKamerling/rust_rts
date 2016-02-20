class BasicUnit extends Unit {
    private wpn_facing: number;
    private wpn_anim: number;

    constructor(c: Cereal, frame: number) {
        super(c, frame);
        this.wpn_facing = c.getU8() * 2 * Math.PI / 255;
        this.wpn_anim = c.getU8();
    }

    getSightRadius(): number {
        return 12;
    }

    getRadius(): number {
        return 8;
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: BasicUnit, time: number, f: number, x: number, y: number): void {
        game.imageer.drawCentered(ctx, "b_unit", 0, f, x, y);
        let wpn_f = Misc.turnTowards(old.wpn_facing, this.wpn_facing, Misc.angularDistance(old.wpn_facing, this.wpn_facing) * time);
        game.imageer.drawCentered(ctx, "b_wpn", 0, wpn_f, x, y);
    }
}