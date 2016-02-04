class Basic extends Unit {
    constructor(c: Cereal, frame: number) {
        super(c, frame);
    }

    getSightRadius(): number {
        return 12;
    }

    getRadius(): number {
        return 8;
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: Unit, time: number, f: number, x: number, y: number): void {
        game.imageer.drawCentered(ctx, "basic", 0, f, x, y);
    }
}