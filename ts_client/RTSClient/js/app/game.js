var Game = (function () {
    function Game() {
        this.imageer = null;
        this.chef = null;
        this.tilemap = null;
        this.control = new DoingNothing();
        this.camera = new Camera(0, 0);
        this.actorCanvas = null;
        this.tilemapCanvas = null;
        this.fogOfWarCanvas = null;
        this.redrawTilemap = true;
        this.connection = null;
        this.actors = null;
        this.logic_frame = 0;
        this.time_since_last_logic_frame = 0;
        this.actors = Array();
        for (var i = 0; i < 2048; i++) {
            this.actors.push(null);
        }
    }
    Game.prototype.setImageer = function (img) {
        this.imageer = img;
    };
    Game.prototype.setChef = function (chef) {
        this.chef = chef;
    };
    Game.prototype.setTilemap = function (tilemap) {
        this.tilemap = tilemap;
    };
    Game.prototype.setTilemapCanvas = function (canvas) {
        this.tilemapCanvas = canvas;
    };
    Game.prototype.setActorCanvas = function (canvas) {
        this.actorCanvas = canvas;
    };
    Game.prototype.setFogOfWarCanvas = function (canvas) {
        this.fogOfWarCanvas = canvas;
    };
    Game.prototype.setConnection = function (conn) {
        this.connection = conn;
    };
    Game.prototype.processPacket = function (data) {
        var logic_frame = data.getU32();
        if (logic_frame > this.logic_frame) {
            this.logic_frame = logic_frame;
            this.time_since_last_logic_frame = 0;
        }
        while (!data.empty()) {
            var msg_type = data.getU8();
            msg_switch: switch (msg_type) {
                case 0:
                    var unit_type = data.getU8();
                    var new_unit = null;
                    unit_switch: switch (unit_type) {
                        case 0:
                            new_unit = new Basic(data, logic_frame);
                            break unit_switch;
                        default:
                            console.log("No unit of type " + unit_type + " exists.");
                            break unit_switch;
                    }
                    // If unit_soul exists, update it with new_unit
                    if (new_unit) {
                        var soul = this.actors[new_unit.unit_ID];
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
    };
    Game.prototype.interactCanvas = function () {
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
    };
    Game.prototype.draw = function (time_passed) {
        this.time_since_last_logic_frame += time_passed;
        this.drawTilemap();
        this.drawActors();
        this.drawFogOfWar();
    };
    Game.prototype.drawTilemap = function () {
        var content = document.getElementById('content');
        if (this.tilemapCanvas.width != content.offsetWidth || this.tilemapCanvas.height != content.offsetHeight) {
            this.tilemapCanvas.width = content.offsetWidth;
            this.tilemapCanvas.height = content.offsetHeight;
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
        var tile = null;
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
    };
    Game.prototype.drawActors = function () {
        var content = document.getElementById('content');
        if (this.actorCanvas.width != content.offsetWidth || this.actorCanvas.height != content.offsetHeight) {
            this.actorCanvas.width = content.offsetWidth;
            this.actorCanvas.height = content.offsetHeight;
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
    };
    Game.prototype.drawFogOfWar = function () {
        var content = document.getElementById('content');
        if (this.fogOfWarCanvas.width != content.offsetWidth || this.fogOfWarCanvas.height != content.offsetHeight) {
            this.fogOfWarCanvas.width = content.offsetWidth;
            this.fogOfWarCanvas.height = content.offsetHeight;
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
    };
    Game.TILESIZE = 32;
    return Game;
})();
var DoingNothing = (function () {
    function DoingNothing() {
    }
    return DoingNothing;
})();
var MovingCamera = (function () {
    function MovingCamera(mx, my, cx, cy) {
        this.clickX = mx;
        this.clickY = my;
        this.cameraX = cx;
        this.cameraY = cy;
    }
    return MovingCamera;
})();
var Camera = (function () {
    function Camera(x, y) {
        this.x = x;
        this.y = y;
    }
    return Camera;
})();
//# sourceMappingURL=game.js.map