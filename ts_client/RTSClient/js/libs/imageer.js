var Imageer = (function () {
    function Imageer(texs, callback) {
        this.sprites = {};
        var that = this;
        var loaded = 0;
        if (texs.length === 0) {
            console.log('No texture(s) specified.');
            callback(this);
        }
        else {
            console.log('Loading textures...');
            for (var i = 0; i < texs.length; i++) {
                var img = new Image();
                img.src = texs[i].url;
                this.sprites[texs[i].name] = { img: img, anim_count: texs[i].anim_count };
                img.onerror = (function (i) {
                    return function (e) {
                        console.log('Failed to load ' + texs[i].url);
                    };
                })(i);
                img.onload = (function (i) {
                    return function (e) {
                        console.log('Loaded ' + texs[i].url);
                        loaded++;
                        if (loaded === texs.length) {
                            callback(that);
                        }
                        ;
                    };
                })(i);
            }
        }
        var circle = document.createElement("canvas");
        circle.width = Imageer.CIRCLE_RADIUS * 2;
        circle.height = Imageer.CIRCLE_RADIUS * 2;
        var ctx = circle.getContext("2d");
        ctx.save();
        ctx.beginPath();
        ctx.fillStyle = '#000000';
        ctx.arc(Imageer.CIRCLE_RADIUS, Imageer.CIRCLE_RADIUS, Imageer.CIRCLE_RADIUS, 0, 2 * Math.PI, true);
        ctx.fill();
        ctx.restore();
        this.circle = convertCanvasToImage(circle);
    }
    Imageer.prototype.drawCentered = function (ctx, name, animN, angle, x, y) {
        var s = this.sprites[name];
        if (s) {
            var img = s.img;
            var n = animN % s.anim_count;
            var sw = img.width / s.anim_count;
            var sx = sw * n;
            var sh = img.height;
            var sy = 0;
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(angle);
            ctx.drawImage(img, sx, sy, sw, sh, -(sw / 2), -(sh / 2), sw, sh);
            ctx.restore();
        }
        else {
            /*
            ctx.save();
            ctx.beginPath();
            ctx.arc(x, y, 16, 0, 2 * Math.PI, false);
            ctx.fillStyle = '#33000099';
            ctx.fill();
            //ctx.lineWidth = 5;
            //ctx.strokeStyle = '#330000';
            //ctx.stroke();
            ctx.restore();
            */
            ctx.save();
            ctx.drawImage(this.circle, x - Imageer.CIRCLE_RADIUS, y - Imageer.CIRCLE_RADIUS);
            ctx.restore();
        }
    };
    Imageer.CIRCLE_RADIUS = 16;
    return Imageer;
})();
//# sourceMappingURL=imageer.js.map