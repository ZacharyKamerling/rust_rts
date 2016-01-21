var Imageer = (function () {
    function Imageer(callback, texs) {
        this.sprites = {};
        var that = this;
        var loaded = 0;
        if (texs.length === 0) {
            console.log('No texture(s) specified.');
            callback(this);
        } else {
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
    return Imageer;
})();
//# sourceMappingURL=Imageer.js.map
