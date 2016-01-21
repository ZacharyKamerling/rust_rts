class Unit {
    unit_ID: number;
    anim_ID: number;
    team: number;
    x: number;
    y: number;
    facing: number;
    health: number;
    progress: number;
    frame_created: number;
    time_since_last_logic_frame: number;

    constructor(c: Cereal, frame: number) {
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

    getSightRadius(): number {
        throw new Error('getSightRadius() is abstract');
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: Unit, time: number, f: number, x: number, y: number): void {
        game.imageer.drawCentered(ctx, "", 0, f, x, y);
    }

    renderFOW(game: Game, ctx: CanvasRenderingContext2D, old: Unit, time: number, f: number, x: number, y: number): void {
        ctx.beginPath();
        ctx.fillStyle = '#000000';
        ctx.arc(x, y, Game.TILESIZE * this.getSightRadius(), 0, 2 * Math.PI, true);
        ctx.fill();
    }
}