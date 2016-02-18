var Game = (function () {
    function Game() {
        this.imageer = null;
        this.chef = null;
        this.tilemap = null;
        this.control = new DoingNothing();
        this.camera = new Camera(0, 0);
        this.actorCanvas = null;
        this.tilemapCanvas = null;
        this.fowCanvas = new FOWCanvas(0, 0);
        this.redrawTilemap = true;
        this.connection = null;
        this.souls = null;
        this.logic_frame = 0;
        this.team = 0;
        this.time_since_last_logic_frame = 0;
        this.souls = Array();
        for (var i = 0; i < 2048; i++) {
            this.souls.push(null);
        }
    }
    Game.prototype.disconnected = function () {
        for (var i = 0; i < 2048; i++) {
            this.souls[i] = null;
        }
    };
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
    Game.prototype.setConnection = function (conn) {
        this.connection = conn;
    };
    Game.prototype.processPacket = function (data) {
        var logic_frame = data.getU32();
        if (logic_frame > this.logic_frame) {
            this.logic_frame = logic_frame;
            this.time_since_last_logic_frame = 0;
            for (var i = 0; i < this.souls.length; i++) {
                var soul = this.souls[i];
                if (soul && (logic_frame - soul.new.frame_created >= 2)) {
                    this.souls[i] = null;
                }
            }
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
                        var soul = this.souls[new_unit.unit_ID];
                        if (soul) {
                            soul.old = soul.new;
                            soul.new = new_unit;
                            soul.new.is_selected = soul.old.is_selected;
                        }
                        else {
                            this.souls[new_unit.unit_ID] = { old: null, new: new_unit };
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
                    // Select things initiate
                    if (event.btn == MouseButton.Left && event.down) {
                        var x = game.camera.x + event.x - game.actorCanvas.width / 2;
                        var y = game.camera.y + event.y - game.actorCanvas.height / 2;
                        game.control = new SelectingUnits(x, y, x, y);
                    }
                    // Issue move order
                    if (event.btn == MouseButton.Right && event.down) {
                        var selected = new Array();
                        for (var i = 0; i < game.souls.length; i++) {
                            var soul = game.souls[i];
                            if (soul && soul.new.is_selected) {
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
                        for (var i = 0; i < selected.length; i++) {
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
                else if (event instanceof MouseMove) {
                    game.redrawTilemap = true;
                    game.camera.x = control.cameraX + control.clickX - event.x;
                    game.camera.y = control.cameraY + control.clickY - event.y;
                }
            }
            else if (control instanceof SelectingUnits) {
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Left && !event.down) {
                        for (var i = 0; i < game.souls.length; i++) {
                            var soul = game.souls[i];
                            if (soul && soul.new && soul.new.team === game.team) {
                                var x = soul.new.x;
                                var y = soul.new.y;
                                var minX = Math.min(control.clickX, control.currentX);
                                var minY = Math.min(control.clickY, control.currentY);
                                var maxX = Math.max(control.clickX, control.currentX);
                                var maxY = Math.max(control.clickY, control.currentY);
                                if (x >= minX && x <= maxX && y >= minY && y <= maxY) {
                                    soul.new.is_selected = true;
                                }
                                else if (!event.shiftDown) {
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
    };
    Game.prototype.draw = function (time_passed) {
        this.time_since_last_logic_frame += time_passed;
        this.drawTilemap();
        this.drawunits();
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
        var startX = Math.floor((this.camera.x - this.tilemapCanvas.width / 2) / Game.TILESIZE) - 1;
        var startY = Math.floor((this.camera.y - this.tilemapCanvas.height / 2) / Game.TILESIZE) - 1;
        var ctx = this.tilemapCanvas.getContext("2d");
        var tile = null;
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
    };
    Game.prototype.drawunits = function () {
        var content = document.getElementById('content');
        if (this.actorCanvas.width != content.offsetWidth || this.actorCanvas.height != content.offsetHeight) {
            this.actorCanvas.width = content.offsetWidth;
            this.actorCanvas.height = content.offsetHeight;
        }
        var ctx = this.actorCanvas.getContext("2d");
        var xOff = this.actorCanvas.width / 2 - this.camera.x;
        var yOff = this.actorCanvas.height / 2 - this.camera.y;
        ctx.clearRect(0, 0, this.actorCanvas.width, this.actorCanvas.height);
        for (var i = 0; i < this.souls.length; i++) {
            var soul = this.souls[i];
            if (soul && soul.new && soul.old && (soul.new.frame_created - soul.old.frame_created <= 2)) {
                var x = soul.old.x + (soul.new.x - soul.old.x) * this.time_since_last_logic_frame;
                var y = soul.old.y + (soul.new.y - soul.old.y) * this.time_since_last_logic_frame;
                var f = Misc.turnTowards(soul.old.facing, soul.new.facing, Misc.angularDistance(soul.old.facing, soul.new.facing) * this.time_since_last_logic_frame);
                soul.new.render(this, ctx, soul.old, this.time_since_last_logic_frame, f, x + xOff, y + yOff);
            }
        }
    };
    Game.prototype.drawFogOfWar = function () {
        var size_ratio = 0.5;
        var content = document.getElementById('content');
        this.fowCanvas.setWidthAndHeight(content.offsetWidth * size_ratio, content.offsetHeight * size_ratio);
        var xOff = content.offsetWidth / 2 - this.camera.x;
        var yOff = content.offsetHeight / 2 - this.camera.y;
        this.fowCanvas.beginRevealing();
        for (var i = 0; i < this.souls.length; i++) {
            var soul = this.souls[i];
            if (soul && soul.new && soul.old && soul.new.team == this.team) {
                var x = (soul.old.x + (soul.new.x - soul.old.x) * this.time_since_last_logic_frame);
                var y = (soul.old.y + (soul.new.y - soul.old.y) * this.time_since_last_logic_frame);
                var sightRadius = soul.new.getSightRadius();
                this.fowCanvas.revealArea((x + xOff) * size_ratio, (y + yOff) * size_ratio, sightRadius * 32 * size_ratio);
            }
        }
        var ctx = this.actorCanvas.getContext("2d");
        ctx.imageSmoothingEnabled = false;
        this.fowCanvas.paintFOW(ctx);
        ctx.imageSmoothingEnabled = true;
    };
    Game.TILESIZE = 32;
    return Game;
})();
var DoingNothing = (function () {
    function DoingNothing() {
    }
    return DoingNothing;
})();
var SelectingUnits = (function () {
    function SelectingUnits(mx, my, cx, cy) {
        this.clickX = mx;
        this.clickY = my;
        this.currentX = cx;
        this.currentY = cy;
    }
    return SelectingUnits;
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