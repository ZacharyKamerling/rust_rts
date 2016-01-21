class Game {
    public imageer: Imageer = null;
    private chef: Chef = null;
    private tilemap: Tilemap<string> = null;
    private control: Control = new DoingNothing();
    private camera: Camera = new Camera(0, 0);
    private actorCanvas: HTMLCanvasElement = null;
    private tilemapCanvas: HTMLCanvasElement = null;
    private fogOfWarCanvas: HTMLCanvasElement = null;
    private redrawTilemap: boolean = true;
    private connection: WebSocket = null;
    private actors: { old: Unit, new: Unit }[] = null;
    private logic_frame: number = 0;
    private time_since_last_logic_frame: number = 0;
    public static TILESIZE = 32;

    constructor() {
        this.actors = Array();

        for (var i = 0; i < 2048; i++) {
            this.actors.push(null);
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

    public setFogOfWarCanvas(canvas: HTMLCanvasElement) {
        this.fogOfWarCanvas = canvas;
    }

    public setConnection(conn: WebSocket) {
        this.connection = conn;
    }

    public processPacket(data: Cereal): void {
        var logic_frame = data.getU32();

        if (logic_frame > this.logic_frame) {
            this.logic_frame = logic_frame;
            this.time_since_last_logic_frame = 0;
        }

        while (!data.empty()) {
            var msg_type = data.getU8();
            msg_switch:
            switch (msg_type) {
                case 0:
                    var unit_type = data.getU8();
                    var new_unit: Unit = null;

                    unit_switch:
                    switch (unit_type) {
                        case 0:
                            new_unit = new Basic(data, logic_frame);
                            break unit_switch;
                        default:
                            console.log("No unit of type " + unit_type + " exists.");
                            break unit_switch;
                    }

                    // If unit_soul exists, update it with new_unit
                    if (new_unit) {
                        var soul = this.actors[new_unit.unit_ID]

                        if (soul) {
                            soul.old = soul.new;
                            soul.new = new_unit;
                        }
                        else {
                            this.actors[new_unit.unit_ID] = { old: null, new: new_unit };
                        }
                    }
                    break msg_switch;

                default:
                    console.log("No message of type " + msg_type + " exists.");
                    break msg_switch;
            }
        }
    }

    public interactCanvas(): ((e: InputEvent) => void) {
        var game = this;

        return function (event) {
            var control = game.control;

            if (control instanceof DoingNothing) {
                if (event instanceof MousePress) {
                    // Move Camera initiate
                    if (event.btn == MouseButton.Middle && event.down) {
                        game.control = new MovingCamera(event.x, event.y, game.camera.x, game.camera.y);
                    }
                }
            }
            else if (control instanceof MovingCamera) {
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Middle && !event.down) {
                        game.control = new DoingNothing();
                    }
                }
                else if (event instanceof MouseMove) {
                    game.redrawTilemap = true;
                    game.camera.x = control.cameraX + control.clickX - event.x;
                    game.camera.y = control.cameraY + control.clickY - event.y;
                }
            }
        };
    }

    public draw(time_passed: number) {
        this.time_since_last_logic_frame += time_passed;
        this.drawTilemap();
        this.drawActors();
        this.drawFogOfWar();
    }

    private drawTilemap() {
        var content = <HTMLDivElement>document.getElementById('content');

        if (this.tilemapCanvas.width != content.offsetWidth || this.tilemapCanvas.height != content.offsetHeight) {
            this.tilemapCanvas.width = content.offsetWidth;
            this.tilemapCanvas.height = content.offsetHeight
            this.redrawTilemap = true;
        }

        if (!this.redrawTilemap) {
            return;
        }

        var cols = Math.floor(this.tilemapCanvas.width / 32) + 3;
        var rows = Math.floor(this.tilemapCanvas.height / 32) + 3;
        // Index to begin drawing tiles
        var startX = Math.floor((this.camera.x - this.tilemapCanvas.width / 2) / 32) - 1;
        var startY = Math.floor((this.camera.y - this.tilemapCanvas.height / 2) / 32) - 1;
        var ctx = this.tilemapCanvas.getContext("2d");
        var tile: string = null;
        // Offset to draw tiles at
        var xOff = this.tilemapCanvas.width / 2 + 16 - this.camera.x;
        var yOff = this.tilemapCanvas.height / 2 + 16 - this.camera.y;

        ctx.clearRect(0, 0, this.tilemapCanvas.width, this.tilemapCanvas.height);
     
        for (var y = startY; y < (rows + startY); y++) {
            for (var x = startX; x < (cols + startX); x++) {
                tile = this.tilemap.getTile(x, y);
                if (tile) {
                    this.imageer.drawCentered(ctx, tile, 0, 0, x * 32 + xOff, y * 32 + yOff);
                }
            }
        }
        this.redrawTilemap = false;
    }

    private drawActors() {

        var content = <HTMLDivElement>document.getElementById('content');

        if (this.actorCanvas.width != content.offsetWidth || this.actorCanvas.height != content.offsetHeight) {
            this.actorCanvas.width = content.offsetWidth;
            this.actorCanvas.height = content.offsetHeight
        }

        var ctx = this.actorCanvas.getContext("2d");
        var xOff = this.actorCanvas.width / 2 - this.camera.x;
        var yOff = this.actorCanvas.height / 2 - this.camera.y;

        ctx.clearRect(0, 0, this.actorCanvas.width, this.actorCanvas.height);

        for (var i = 0; i < this.actors.length; i++) {
            var soul = this.actors[i];
            if (soul && soul.new && soul.old) {
                var x = soul.old.x + (soul.new.x - soul.old.x) * this.time_since_last_logic_frame;
                var y = soul.old.y + (soul.new.y - soul.old.y) * this.time_since_last_logic_frame;
                var f = soul.new.facing;
                soul.new.render(this, ctx, soul.old, this.time_since_last_logic_frame, f, x + xOff, y + yOff);
            }
        }
    }

    private drawFogOfWar() {
        var content = <HTMLDivElement>document.getElementById('content');

        if (this.fogOfWarCanvas.width != content.offsetWidth || this.fogOfWarCanvas.height != content.offsetHeight) {
            this.fogOfWarCanvas.width = content.offsetWidth;
            this.fogOfWarCanvas.height = content.offsetHeight
        }

        var ctx = this.fogOfWarCanvas.getContext("2d");
        var xOff = this.fogOfWarCanvas.width / 2 - this.camera.x;
        var yOff = this.fogOfWarCanvas.height / 2 - this.camera.y;

        ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
        ctx.fillStyle = "rgba(0, 0, 0, 0.85)";
        ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);

        ctx.save();
        ctx.globalCompositeOperation = 'destination-out';

        for (var i = 0; i < this.actors.length; i++) {
            var soul = this.actors[i];
            if (soul && soul.new && soul.old) {
                var x = soul.old.x + (soul.new.x - soul.old.x) * this.time_since_last_logic_frame;
                var y = soul.old.y + (soul.new.y - soul.old.y) * this.time_since_last_logic_frame;
                var f = soul.new.facing;
                soul.new.renderFOW(this, ctx, soul.old, this.time_since_last_logic_frame, f, x + xOff, y + yOff);
            }
        }
        ctx.restore();
    }
}

interface Control { }

class DoingNothing implements Control { }

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