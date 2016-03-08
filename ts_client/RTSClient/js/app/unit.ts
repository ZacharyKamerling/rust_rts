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
    is_selected: boolean;

    constructor(c: Cereal, frame: number) {
        if (c) {
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
    }

    copycat(unit: Unit) {
        unit.unit_ID = this.unit_ID;
        unit.anim_ID = this.anim_ID;
        unit.team = this.team;
        unit.x = this.x;
        unit.y = this.y;
        unit.facing = this.facing;
        unit.health = this.health;
        unit.progress = this.progress;
        unit.frame_created = this.frame_created;
        unit.is_selected = this.is_selected;
    }

    clone(): Unit {
        throw new Error('Unit: clone() is abstract');
    }

    getSightRadius(): number {
        throw new Error('Unit: getSightRadius() is abstract');
    }

    getRadius(): number {
        throw new Error('Unit: getRadius() is abstract');
    }

    render(game: Game, ctx: CanvasRenderingContext2D, x: number, y: number): void {
        throw new Error('Unit: render() is abstract');
    }

    step(time: number, oldUnit: Unit, newUnit: Unit) {
        let f1 = oldUnit.facing;
        let f2 = newUnit.facing;
        let turn = Misc.angularDistance(f1, f2) * time;
        this.facing = Misc.turnTowards(this.facing, f2, turn);
        this.x = this.x + (newUnit.x - oldUnit.x) * time;
        this.y = this.y + (newUnit.y - oldUnit.y) * time;
        this.health = this.health + (newUnit.health - oldUnit.health) * time;
        this.progress = this.progress + (newUnit.progress - oldUnit.progress) * time;
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