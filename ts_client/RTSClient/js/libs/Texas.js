var Texas = (function () {
    function Texas(callback, width, height, texs) {
        this.spritesheet = new Image();
        this.sprites = {};
        var canv = document.createElement('canvas');
        canv.width = width;
        canv.height = height;
        var ctx = canv.getContext('2d');
        var curW = 0;
        var curH = 0;
        var maxH = 0;
        var loaded = 0;
        var that = this;
        var imgs = new Array(texs.length);
        if (texs.length === 0) {
            this.spritesheet.src = canv.toDataURL("image/png");
            console.log('No texture(s) specified.');
            callback(this);
        }
        else {
            console.log('Loading textures...');
            for (var i = 0; i < texs.length; i++) {
                imgs[i] = new Image();
                imgs[i].src = texs[i].url;
                imgs[i].onerror = (function (i) {
                    return function (e) {
                        console.log('Failed to load ' + texs[i].url);
                    };
                })(i);
                imgs[i].onload = (function (i) {
                    return function (e) {
                        console.log('Loaded ' + texs[i].url);
                        loaded++;
                        if (loaded === texs.length) {
                            for (var n = 0; n < texs.length; n++) {
                                // If image wider than space left in row, create new row
                                if (curW + texs[n].w >= width) {
                                    curW = 0;
                                    curH += maxH;
                                    maxH = 0;
                                }
                                // If image taller than all other images in row, set new max height for row
                                if (texs[n].h > maxH) {
                                    maxH = texs[n].h;
                                }
                                ctx.drawImage(imgs[n], 0, 0, imgs[n].naturalWidth, imgs[n].naturalHeight, curW, curH, texs[n].w, texs[n].h);
                                that.sprites[(texs[n].name)] = { x: curW, y: curH, w: texs[n].w, h: texs[n].h, n: texs[n].anims };
                                curW += texs[n].w;
                            }
                            that.spritesheet.src = canv.toDataURL("image/png");
                            callback(that);
                        }
                        ;
                    };
                })(i);
            }
        }
    }
    Texas.prototype.drawCentered = function (ctx, name, animN, x, y) {
        var s = this.sprites[name];
        if (s) {
            var n = animN % s.n;
            var sw = s.w / s.n;
            var sx = s.x + sw * n;
            var sh = s.h;
            var sy = s.y;
            ctx.drawImage(this.spritesheet, sx, sy, sw, sh, x - (sw / 2), y - (sh / 2), sw, sh);
        }
        else {
            ctx.save();
            ctx.fillStyle = name;
            ctx.fillRect(x - 32, y - 32, 32, 32);
            ctx.restore();
        }
    };
    return Texas;
})();
//# sourceMappingURL=Texas.js.map