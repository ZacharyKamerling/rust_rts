define(["require", "exports", "libs/interact"], function (require, exports, I) {
    var Game = (function () {
        function Game() {
            this.imageer = null;
            this.chef = null;
            this.tilemap = null;
            this.control = null;
            this.camera = null;
            this.actorCanvas = null;
            this.tilemapCanvas = null;
            this.redrawTilemap = true;
            this.control = new DoingNothing();
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
        Game.prototype.interactCanvas = function () {
            var game = this;
            var control = game.control;
            return function (event) {
                if (control instanceof DoingNothing) {
                    if (event instanceof I.MousePress) {
                        // Move Camera initiate
                        if (event.btn == I.MouseButton.Middle && event.down) {
                            game.control = new MovingCamera(event.x, event.y, game.camera.x, game.camera.y);
                        }
                    }
                }
                else if (control instanceof MovingCamera) {
                    if (event instanceof I.MousePress) {
                        if (event.btn == I.MouseButton.Middle && !event.down) {
                            game.control = new DoingNothing();
                        }
                        else if (event instanceof I.MouseMove) {
                            var cw = game.tilemapCanvas.width;
                            var ch = game.tilemapCanvas.height;
                            var mx = event.x;
                            var my = ch - event.y;
                            game.camera.x = control.cameraX + control.clickX - mx;
                            game.camera.y = control.cameraY + control.clickY - my;
                        }
                    }
                }
            };
        };
        return Game;
    })();
    exports.Game = Game;
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
});
//# sourceMappingURL=game.js.map