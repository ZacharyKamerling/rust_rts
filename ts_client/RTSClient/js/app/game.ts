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
    private units: { old: Unit, new: Unit }[] = null;
    private logic_frame: number = 0;
    private team: number = 0;
    private time_since_last_logic_frame: number = 0;
    private fog_of_war_sprite: HTMLImageElement;
    public static TILESIZE = 32;

    constructor() {
        // Fog of war sprite
        var fows = document.createElement("canvas");
        fows.width = 8 * 2 * Game.TILESIZE;
        fows.height = 8 * 2 * Game.TILESIZE;
        var ctx = fows.getContext("2d");

        ctx.save();
        ctx.beginPath();
        ctx.fillStyle = '#000000';
        ctx.arc(8 * Game.TILESIZE, 8 * Game.TILESIZE, Game.TILESIZE * 8, 0, 2 * Math.PI, true);
        ctx.fill();
        ctx.restore();

        this.units = Array();
        this.fog_of_war_sprite = convertCanvasToImage(fows);

        for (var i = 0; i < 2048; i++) {
            this.units.push(null);
        }
    }

    public disconnected() {
        for (var i = 0; i < 2048; i++) {
            this.units[i] = null;
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
                        var soul = this.units[new_unit.unit_ID]

                        if (soul) {
                            soul.old = soul.new;
                            soul.new = new_unit;
                            soul.new.is_selected = soul.old.is_selected;
                        }
                        else {
                            this.units[new_unit.unit_ID] = { old: null, new: new_unit };
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
                    // Select things initiate
                    if (event.btn == MouseButton.Left && event.down) {
                        var x = game.camera.x + event.x - game.actorCanvas.width / 2;
                        var y = game.camera.y + event.y - game.actorCanvas.height / 2;
                        game.control = new SelectingUnits(x, y, x, y);
                    }
                    // Issue move order
                    if (event.btn == MouseButton.Right && event.down) {
                        var selected: number[] = new Array();

                        for (var i = 0; i < game.units.length; i++) {
                            var soul = game.units[i];
                            if (soul && soul.new.is_selected) {
                                selected.push(i);
                            }
                        }

                        game.chef.put8(0);
                        game.chef.put8(0);
                        game.chef.put16(selected.length);
                        game.chef.putF64((game.camera.x + event.x - game.actorCanvas.width / 2) / Game.TILESIZE);
                        game.chef.putF64((game.camera.y + event.y - game.actorCanvas.height / 2) / Game.TILESIZE);

                        for (var i = 0; i < selected.length; i++) {
                            game.chef.put16(selected[i]);
                        }

                        game.connection.send(game.chef.done());
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
            else if (control instanceof SelectingUnits) {
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Left && !event.down) {

                        for (var i = 0; i < game.units.length; i++) {
                            var soul = game.units[i];

                            if (soul && soul.new && soul.new.team == game.team) {
                                var x = soul.new.x;
                                var y = soul.new.y;
                                var minX = Math.min(control.clickX, control.currentX);
                                var minY = Math.min(control.clickY, control.currentY);
                                var maxX = Math.max(control.clickX, control.currentX);
                                var maxY = Math.max(control.clickY, control.currentY);

                                if (x >= minX && x <= maxX && y >= minY && y <= maxY) {
                                    soul.new.is_selected = true;
                                }
                                else {
                                    soul.new.is_selected = false;
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
        this.drawTilemap();
        this.drawunits();
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
        var startX = Math.floor((this.camera.x - this.tilemapCanvas.width / 2) / Game.TILESIZE) - 1;
        var startY = Math.floor((this.camera.y - this.tilemapCanvas.height / 2) / Game.TILESIZE) - 1;
        var ctx = this.tilemapCanvas.getContext("2d");
        var tile: string = null;
        // Offset to draw tiles at
        var xOff = this.tilemapCanvas.width / 2 + (Game.TILESIZE / 2) - this.camera.x;
        var yOff = this.tilemapCanvas.height / 2 + (Game.TILESIZE / 2) - this.camera.y;

        ctx.clearRect(0, 0, this.tilemapCanvas.width, this.tilemapCanvas.height);
     
        for (var y = startY; y < (rows + startY); y++) {
            for (var x = startX; x < (cols + startX); x++) {
                tile = this.tilemap.getTile(x, y);
                if (tile) {
                    this.imageer.drawCentered(ctx, tile, 0, 0, x * Game.TILESIZE + xOff, y * Game.TILESIZE + yOff);
                }
            }
        }
        this.redrawTilemap = false;
    }

    private drawunits() {

        var content = <HTMLDivElement>document.getElementById('content');

        if (this.actorCanvas.width != content.offsetWidth || this.actorCanvas.height != content.offsetHeight) {
            this.actorCanvas.width = content.offsetWidth;
            this.actorCanvas.height = content.offsetHeight
        }

        var ctx = this.actorCanvas.getContext("2d");
        var xOff = this.actorCanvas.width / 2 - this.camera.x;
        var yOff = this.actorCanvas.height / 2 - this.camera.y;

        ctx.clearRect(0, 0, this.actorCanvas.width, this.actorCanvas.height);

        for (var i = 0; i < this.units.length; i++) {
            var soul = this.units[i];
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

        for (var i = 0; i < this.units.length; i++) {
            var soul = this.units[i];
            if (soul && soul.new && soul.old) {
                var x = soul.old.x + (soul.new.x - soul.old.x) * this.time_since_last_logic_frame;
                var y = soul.old.y + (soul.new.y - soul.old.y) * this.time_since_last_logic_frame;
                var sightRadius = soul.new.getSightRadius();

                ctx.drawImage(this.fog_of_war_sprite, x + xOff - (sightRadius * 32), y + yOff - (sightRadius * 32), sightRadius * 64, sightRadius * 64);
            }
        }
        ctx.restore();
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