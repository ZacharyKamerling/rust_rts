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
    time_created: number;
    is_selected: boolean;

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
        throw new Error('Unit: getSightRadius() is abstract');
    }

    getRadius(): number {
        throw new Error('Unit: getRadius() is abstract');
    }

    render(game: Game, ctx: CanvasRenderingContext2D, old: Unit, time: number, f: number, x: number, y: number): void {
        throw new Error('Unit: render() is abstract');
    }

    static decodeUnit(data: Cereal, frame: number): Unit {
        let unitType = data.getU8();
        switch (unitType) {
            case 0:
                return new BasicUnit(data, frame);
            default:
                console.log("No unit of type " + unitType + " exists.");
                return null;
        }
    }
}