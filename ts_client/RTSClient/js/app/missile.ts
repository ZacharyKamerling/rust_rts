class Missile {
    misl_ID: number;
    x: number;
    y: number;
    exploding: boolean;
    frame_created: number;
    time_created: number;

    constructor(c: Cereal, frame: number, exploding: boolean) {
        this.frame_created = frame;
        this.exploding = exploding;
        this.misl_ID = c.getU16();
        this.x = c.getU16() / 2;
        this.y = c.getU16() / 2;
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: Missile, time: number, f: number, x: number, y: number): void {
        throw new Error('Missile: render() is abstract');
    }

    speed(): number {
        throw new Error('Missile: speed() is abstract');
    }

    static decodeMissile(data: Cereal, frame: number, exploding: boolean): Missile {
        let mislType = data.getU8();
        switch (mislType) {
            case 0:
                return new BasicMissile(data, frame, exploding);
            default:
                console.log("No missile of type " + mislType + " exists.");
                return null;
        }
    }
}