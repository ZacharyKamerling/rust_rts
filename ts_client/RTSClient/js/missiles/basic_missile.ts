class BasicMissile extends Missile{
    constructor(c: Cereal, frame: number, exploding: boolean) {
        super(c, frame, exploding);
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: Missile, time: number, f: number, x: number, y: number): void {
        game.imageer.drawCentered(ctx, "b_misl", 0, f, x, y);
    }

    renderExplosion(game: Game, ctx: CanvasRenderingContext2D, old: Missile, time: number, f: number, x: number, y: number): void {
        game.imageer.drawCentered(ctx, "b_misl", 0, f, x, y);
    }

    speed(): number {
        return Game.TILESIZE * 10.0 / 10.0;
    }
}