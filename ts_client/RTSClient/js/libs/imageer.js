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
            ctx.save();
            ctx.beginPath();
            ctx.arc(x, y, 16, 0, 2 * Math.PI, false);
            ctx.fillStyle = '#33000099';
            ctx.fill();
            //ctx.lineWidth = 5;
            //ctx.strokeStyle = '#330000';
            //ctx.stroke();
            ctx.restore();
        }
    };
    return Imageer;
})();
//# sourceMappingURL=imageer.js.map