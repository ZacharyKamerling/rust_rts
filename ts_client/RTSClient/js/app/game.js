"use strict";
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
        this.missile_souls = null;
        this.logic_frame = 0;
        this.team = 0;
        this.time_since_last_logic_frame = 0;
        this.souls = Array();
        for (var i = 0; i < 2048; i++) {
            this.souls.push(null);
        }
        this.missile_souls = Array();
        for (var i = 0; i < 2048 * 2; i++) {
            this.missile_souls.push(null);
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
        if (logic_frame >= this.logic_frame) {
            this.logic_frame = logic_frame;
            this.time_since_last_logic_frame = 0;
            for (var i = 0; i < this.souls.length; i++) {
                var soul = this.souls[i];
                if (soul && (logic_frame - soul.new.frame_created >= 2)) {
                    this.souls[i] = null;
                }
            }
            for (var i = 0; i < this.missile_souls.length; i++) {
                var misl_soul = this.missile_souls[i];
                if (misl_soul && (logic_frame - misl_soul.new.frame_created >= 2)) {
                    this.missile_souls[i] = null;
                }
            }
        }
        else {
            return;
        }
        while (!data.empty()) {
            var msg_type = data.getU8();
            msg_switch: switch (msg_type) {
                // Unit
                case 0:
                    var new_unit = Unit.decodeUnit(data, logic_frame);
                    // If unit_soul exists, update it with new_unit
                    if (new_unit) {
                        var soul = this.souls[new_unit.unit_ID];
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
                    var exploding = msg_type === 2;
                    var new_misl = Missile.decodeMissile(data, logic_frame, exploding);
                    if (new_misl) {
                        var soul = this.missile_souls[new_misl.misl_ID];
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
                    var unit_ID = data.getU16();
                    var dmg_type = data.getU8();
                    this.souls[unit_ID] = null;
                    break msg_switch;
                default:
                    console.log("No message of type " + msg_type + " exists.");
                    return;
            }
        }
    };
    Game.prototype.interact_canvas = function () {
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
                // Select units
                if (event instanceof MousePress) {
                    if (event.btn == MouseButton.Left && !event.down) {
                        for (var i = 0; i < game.souls.length; i++) {
                            var soul = game.souls[i];
                            if (soul && soul.new && soul.new.team === game.team) {
                                var x = soul.current.x;
                                var y = soul.current.y;
                                var minX = Math.min(control.clickX, control.currentX);
                                var minY = Math.min(control.clickY, control.currentY);
                                var maxX = Math.max(control.clickX, control.currentX);
                                var maxY = Math.max(control.clickY, control.currentY);
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
    };
    Game.prototype.draw = function (time_passed) {
        this.time_since_last_logic_frame += time_passed;
        this.stepUnits(time_passed);
        this.drawTilemap();
        this.drawunits();
        this.drawFogOfWar();
    };
    Game.prototype.stepUnits = function (time) {
        for (var i = 0; i < this.souls.length; i++) {
            var soul = this.souls[i];
            if (soul && soul.current && soul.new && soul.old) {
                soul.current.step(time, soul.old, soul.new);
            }
        }
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
        {
            for (var i = 0; i < this.souls.length; i++) {
                var soul = this.souls[i];
                if (soul && soul.new && soul.old) {
                    var x = soul.current.x + xOff;
                    var y = soul.current.y + yOff;
                    soul.current.render(this, ctx, x, y);
                }
            }
        }
        for (var i = 0; i < this.missile_souls.length; i++) {
            var soul = this.missile_souls[i];
            if (soul && soul.new && soul.old) {
                var f = Math.atan2(soul.new.y - soul.old.y, soul.new.x - soul.old.x);
                var coeff = this.time_since_last_logic_frame + (this.logic_frame - soul.new.frame_created);
                var x = soul.old.x + soul.new.speed() * Math.cos(f) * coeff;
                var y = soul.old.y + soul.new.speed() * Math.sin(f) * coeff;
                soul.new.render(this, ctx, soul.old, coeff, f, x + xOff, y + yOff);
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
                var x = soul.current.x;
                var y = soul.current.y;
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