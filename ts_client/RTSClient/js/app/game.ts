"use strict";
class Game {
    public imageer: Imageer = null;
    private chef: Chef = null;
    public tilemap: Tilemap<string> = null;
    private control: Control = new DoingNothing();
    private camera: Camera = new Camera(0, 0);
    private actorCanvas: HTMLCanvasElement = null;
    private tilemapCanvas: HTMLCanvasElement = null;
    private fowCanvas: FOWCanvas = new FOWCanvas(0, 0);
    private redrawTilemap: boolean = true;
    private connection: WebSocket = null;
    private souls: { old: Unit, current: Unit, new: Unit }[] = null;
    private missile_souls: { old: Missile, new: Missile }[] = null;
    private logic_frame: number = 0;
    private team: number = 0;
    private time_since_last_logic_frame: number = 0;
    public static TILESIZE = 32;

    constructor() {
        this.souls = Array();

        for (let i = 0; i < 2048; i++) {
            this.souls.push(null);
        }

        this.missile_souls = Array();

        for (let i = 0; i < 2048 * 2; i++) {
            this.missile_souls.push(null);
        }
    }

    public disconnected() {
        for (let i = 0; i < 2048; i++) {
            this.souls[i] = null;
        }
    }

    public setImageer(img: Imageer) {
        this.imageer = img;
    }

    public setChef(chef: Chef) {
        this.chef = chef;
    }

    public setTilemap(tilemap: Tilemap<string>) {
        this.tilemap = tilemap;
    }

    public setTilemapCanvas(canvas: HTMLCanvasElement) {
        this.tilemapCanvas = canvas;
    }

    public setActorCanvas(canvas: HTMLCanvasElement) {
        this.actorCanvas = canvas;
    }

    public setConnection(conn: WebSocket) {
        this.connection = conn;
    }

    public processPacket(data: Cereal): void {
        let logic_frame = data.getU32();

        if (logic_frame >= this.logic_frame) {
            this.logic_frame = logic_frame;
            this.time_since_last_logic_frame = 0;

            for (let i = 0; i < this.souls.length; i++) {
                let soul = this.souls[i];
                if (soul && (logic_frame - soul.new.frame_created >= 2)) {
                    this.souls[i] = null;
                }
            }

            for (let i = 0; i < this.missile_souls.length; i++) {
                let misl_soul = this.missile_souls[i];
                if (misl_soul && (logic_frame - misl_soul.new.frame_created >= 2)) {
                    this.missile_souls[i] = null;
                }
            }
        }
        else {
            return;
        }

        while (!data.empty()) {
            let msg_type = data.getU8();


            msg_switch:
            switch (msg_type) {
                // Unit
                case 0:
                    let new_unit: Unit = Unit.decodeUnit(data, logic_frame);

                    // If unit_soul exists, update it with new_unit
                    if (new_unit) {
                        let soul = this.souls[new_unit.unit_ID];

                        if (soul) {
                            soul.old = soul.current.clone();
                            soul.new = new_unit;
                        }
                        else {
                            var cur = new_unit.clone();
                            this.souls[new_unit.unit_ID] = { old: null, current: cur, new: new_unit };
                        }
                    }
                    break msg_switch;
                // Missile
                case 1:
                case 2:
                    let exploding = msg_type === 2;
                    let new_misl: Missile = Missile.decodeMissile(data, logic_frame, exploding);

                    if (new_misl) {
                        let soul = this.missile_souls[new_misl.misl_ID];

                        if (soul) {
                            soul.old = soul.new;
                            soul.new = new_misl;
                        }
                        else {
                            this.missile_souls[new_misl.misl_ID] = { old: null, new: new_misl };
                        }
                    }
                    break msg_switch;
                // Unit death
                case 3:
                    let unit_ID = data.getU16();
                    let dmg_type = data.getU8();
                    this.souls[unit_ID] = null;
                    break msg_switch;
                default:
                    console.log("No message of type " + msg_type + " exists.");
                    return;
            }
        }
    }

    public interact_canvas(): ((e: InputEvent) => void) {
        let game = this;

        return function (event) {
            let control = game.control;

            if (control instanceof DoingNothing) {
                if (event instanceof MousePress) {
                    // Move Camera initiate
                    if (event.btn == MouseButton.Middle && event.down) {
                        game.control = new MovingCamera(event.x, event.y, game.camera.x, game.camera.y);
                    }
                    // Select things initiate
                    if (event.btn == MouseButton.Left && event.down) {
                        let x = game.camera.x + event.x - game.actorCanvas.width / 2;
                        let y = game.camera.y + event.y - game.actorCanvas.height / 2;
                        game.control = new SelectingUnits(x, y, x, y);
                    }
                    // Issue move order
                    if (event.btn == MouseButton.Right && event.down) {
                        let selected: number[] = new Array();

                        for (let i = 0; i < game.souls.length; i++) {
                            let soul = game.souls[i];

                            if (soul && soul.current.is_selected) {
                                selected.push(i);
                            }
                        }

                        game.chef.put8(0);
                        if (event.shiftDown) {
                            game.chef.put8(1);
                        }
                        else {
                            game.chef.put8(0);
                        }
                        
                        game.chef.put16(selected.length);
                        game.chef.putF64((game.camera.x + event.x - game.actorCanvas.width / 2) / Game.TILESIZE);
                        game.chef.putF64((game.camera.y + event.y - game.actorCanvas.height / 2) / Game.TILESIZE);

                        for (let i = 0; i < selected.length; i++) {
                            game.chef.put16(selected[i]);
                        }
                        game.connection.send(game.chef.done());
                    }
                }
            }
            else if (control instanceof MovingCamera) {
                // Stop moving camera
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Middle && !event.down) {
                        game.control = new DoingNothing();
                    }
                }
                // Move camera
                else if (event instanceof MouseMove) {
                    game.redrawTilemap = true;
                    game.camera.x = control.cameraX + control.clickX - event.x;
                    game.camera.y = control.cameraY + control.clickY - event.y;
                }
            }
            else if (control instanceof SelectingUnits) {
                // Select units
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Left && !event.down) {

                        for (let i = 0; i < game.souls.length; i++) {
                            let soul = game.souls[i];

                            if (soul && soul.new && soul.new.team === game.team) {
                                let x = soul.current.x;
                                let y = soul.current.y;
                                let minX = Math.min(control.clickX, control.currentX);
                                let minY = Math.min(control.clickY, control.currentY);
                                let maxX = Math.max(control.clickX, control.currentX);
                                let maxY = Math.max(control.clickY, control.currentY);

                                if (x >= minX && x <= maxX && y >= minY && y <= maxY) {
                                    soul.current.is_selected = true;
                                }
                                else if (!event.shiftDown) {
                                    soul.current.is_selected = false;
                                }
                            }
                        }

                        game.control = new DoingNothing();
                    }
                }
                else if (event instanceof MouseMove) {
                    game.redrawTilemap = true;
                    control.currentX = game.camera.x + event.x - game.actorCanvas.width / 2;
                    control.currentY = game.camera.y + event.y - game.actorCanvas.height / 2;
                }
            }
        };
    }

    public draw(time_passed: number) {
        this.time_since_last_logic_frame += time_passed;
        this.stepUnits(time_passed);
        this.drawTilemap();
        this.drawunits();
        this.drawFogOfWar();
    }

    private stepUnits(time: number) {
        for (let i = 0; i < this.souls.length; i++) {
            var soul = this.souls[i];
            if (soul && soul.current && soul.new && soul.old) {
                soul.current.step(time, soul.old, soul.new);
            }
        }
    }

    private drawTilemap() {
        let content = <HTMLDivElement>document.getElementById('content');

        if (this.tilemapCanvas.width != content.offsetWidth || this.tilemapCanvas.height != content.offsetHeight) {
            this.tilemapCanvas.width = content.offsetWidth;
            this.tilemapCanvas.height = content.offsetHeight
            this.redrawTilemap = true;
        }

        if (!this.redrawTilemap) {
            return;
        }

        let cols = Math.floor(this.tilemapCanvas.width / 32) + 3;
        let rows = Math.floor(this.tilemapCanvas.height / 32) + 3;
        // Index to begin drawing tiles
        let startX = Math.floor((this.camera.x - this.tilemapCanvas.width / 2) / Game.TILESIZE) - 1;
        let startY = Math.floor((this.camera.y - this.tilemapCanvas.height / 2) / Game.TILESIZE) - 1;
        let ctx = this.tilemapCanvas.getContext("2d");
        let tile: string = null;
        // Offset to draw tiles at
        let xOff = this.tilemapCanvas.width / 2 + (Game.TILESIZE / 2) - this.camera.x;
        let yOff = this.tilemapCanvas.height / 2 + (Game.TILESIZE / 2) - this.camera.y;

        ctx.clearRect(0, 0, this.tilemapCanvas.width, this.tilemapCanvas.height);
     
        for (let y = startY; y < (rows + startY); y++) {
            for (let x = startX; x < (cols + startX); x++) {
                tile = this.tilemap.getTile(x, y);
                if (tile) {
                    this.imageer.drawCentered(ctx, tile, 0, 0, x * Game.TILESIZE + xOff, y * Game.TILESIZE + yOff);
                }
            }
        }
        this.redrawTilemap = false;
    }

    private drawunits() {

        let content = <HTMLDivElement>document.getElementById('content');

        if (this.actorCanvas.width != content.offsetWidth || this.actorCanvas.height != content.offsetHeight) {
            this.actorCanvas.width = content.offsetWidth;
            this.actorCanvas.height = content.offsetHeight;
        }

        let ctx = this.actorCanvas.getContext("2d");
        let xOff = this.actorCanvas.width / 2 - this.camera.x;
        let yOff = this.actorCanvas.height / 2 - this.camera.y;

        ctx.clearRect(0, 0, this.actorCanvas.width, this.actorCanvas.height);
        {
            for (let i = 0; i < this.souls.length; i++) {
                let soul = this.souls[i];
                if (soul && soul.new && soul.old) {
                    let x = soul.current.x + xOff;
                    let y = soul.current.y + yOff;
                    soul.current.render(this, ctx, x, y);
                }
            }
        }

        for (let i = 0; i < this.missile_souls.length; i++) {
            let soul = this.missile_souls[i];

            if (soul && soul.new && soul.old) {
                let f = Math.atan2(soul.new.y - soul.old.y, soul.new.x - soul.old.x);
                let coeff = this.time_since_last_logic_frame + (this.logic_frame - soul.new.frame_created);
                let x = soul.old.x + soul.new.speed() * Math.cos(f) * coeff;
                let y = soul.old.y + soul.new.speed() * Math.sin(f) * coeff;
                soul.new.render(this, ctx, soul.old, coeff, f, x + xOff, y + yOff);
            }
        }
    }

    private drawFogOfWar() {
        let size_ratio = 0.5;
        let content = <HTMLDivElement>document.getElementById('content');

        this.fowCanvas.setWidthAndHeight(content.offsetWidth * size_ratio, content.offsetHeight * size_ratio);
        let xOff = content.offsetWidth / 2 - this.camera.x;
        let yOff = content.offsetHeight / 2 - this.camera.y;

        this.fowCanvas.beginRevealing();

        for (let i = 0; i < this.souls.length; i++) {
            let soul = this.souls[i];
            if (soul && soul.new && soul.old && soul.new.team == this.team) {
                let x = soul.current.x;
                let y = soul.current.y;
                let sightRadius = soul.new.getSightRadius();

                this.fowCanvas.revealArea((x + xOff) * size_ratio, (y + yOff) * size_ratio, sightRadius * 32 * size_ratio);
            }
        }
        let ctx: any = this.actorCanvas.getContext("2d");
        ctx.imageSmoothingEnabled = false;
        this.fowCanvas.paintFOW(ctx);
        ctx.imageSmoothingEnabled = true;
    }
}

interface Control { }

class DoingNothing implements Control { }

class SelectingUnits implements Control {
    clickX: number;
    clickY: number;
    currentX: number;
    currentY: number;

    constructor(mx: number, my: number, cx: number, cy: number) {
        this.clickX = mx;
        this.clickY = my;
        this.currentX = cx;
        this.currentY = cy;
    }
}

class MovingCamera implements Control {
    clickX: number;
    clickY: number;
    cameraX: number;
    cameraY: number;

    constructor(mx: number, my: number, cx: number, cy: number) {
        this.clickX = mx;
        this.clickY = my;
        this.cameraX = cx;
        this.cameraY = cy;
    }
}

class Camera {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }
}