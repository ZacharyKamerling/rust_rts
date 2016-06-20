class TileDrawer {
    private canvas: HTMLCanvasElement;
    private ctx: WebGLRenderingContext;
    private buffer: WebGLBuffer;
    private program: MetaProgram;
    private tileTexture: WebGLTexture;
    private spriteSheet: WebGLTexture;
    private tileSize: number;
    private spriteSheetScaleX: number;
    private spriteSheetScaleY: number;
    private tileTextureScaleX: number;
    private tileTextureScaleY: number;

    constructor(canvas: HTMLCanvasElement, spriteSrc: string, tileSrc: string) {
        this.canvas = canvas
        let gl = <WebGLRenderingContext>this.canvas.getContext('webgl');
        this.program = new MetaProgram(gl, createProgram(gl, TileDrawer.vertexShader, TileDrawer.fragmentShader));
        this.tileTexture = gl.createTexture();
        this.spriteSheet = gl.createTexture();
        this.tileSize = 16;

        let self = this;
        let sprts = new Image();
        let tiles = new Image();

        sprts.onerror = function (e: Event) {
            console.log('Failed to load ' + spriteSrc);
        };

        sprts.onload = function (e: Event) {
            console.log('Loaded ' + spriteSrc);
            self.ctx.bindTexture(self.ctx.TEXTURE_2D, self.spriteSheet);
            self.ctx.texImage2D(self.ctx.TEXTURE_2D, 0, self.ctx.RGBA, self.ctx.RGBA, self.ctx.UNSIGNED_BYTE, sprts);
            self.ctx.texParameteri(self.ctx.TEXTURE_2D, self.ctx.TEXTURE_MAG_FILTER, self.ctx.NEAREST);
            self.ctx.texParameteri(self.ctx.TEXTURE_2D, self.ctx.TEXTURE_MIN_FILTER, self.ctx.NEAREST);

            self.spriteSheetScaleX = 1 / sprts.width;
            self.spriteSheetScaleY = 1 / sprts.height;
        };

        sprts.src = spriteSrc;

        tiles.onerror = function (e: Event) {
            console.log('Failed to load ' + tileSrc);
        };

        tiles.onload = function (e: Event) {
            console.log('Loaded ' + tileSrc);
            self.ctx.bindTexture(self.ctx.TEXTURE_2D, self.tileTexture);
            self.ctx.texImage2D(self.ctx.TEXTURE_2D, 0, self.ctx.RGBA, self.ctx.RGBA, self.ctx.UNSIGNED_BYTE, tiles);
            self.ctx.texParameteri(self.ctx.TEXTURE_2D, self.ctx.TEXTURE_MAG_FILTER, self.ctx.NEAREST);
            self.ctx.texParameteri(self.ctx.TEXTURE_2D, self.ctx.TEXTURE_MIN_FILTER, self.ctx.NEAREST);

            self.tileTextureScaleX = 1 / tiles.width;
            self.tileTextureScaleY = 1 / tiles.height;
        };

        tiles.src = tileSrc;

        let buffer = [
            //x  y  u  v
            -1, -1, 0, 1,
             1, -1, 1, 1,
             1,  1, 1, 0,

            -1, -1, 0, 1,
             1,  1, 1, 0,
            -1,  1, 0, 0
        ];

        this.buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(buffer), gl.STATIC_DRAW);
    }

    draw(x: number, y: number, scale: number) {
        let ss = scale * scale;
        x = x / scale - (this.canvas.width / 2) / ss;
        y = y / scale - (this.canvas.height / 2) / ss;

        let gl = <WebGLRenderingContext>this.canvas.getContext('webgl');
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.useProgram(this.program.program);

        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);

        gl.enableVertexAttribArray(this.program.attribute['position']);
        gl.enableVertexAttribArray(this.program.attribute['texture']);
        gl.vertexAttribPointer(this.program.attribute['position'], 2, gl.FLOAT, false, 16, 0);
        gl.vertexAttribPointer(this.program.attribute['texture'], 2, gl.FLOAT, false, 16, 8);

        gl.uniform2f(this.program.uniform['viewportSize'], this.canvas.width / scale, this.canvas.height / scale);
        gl.uniform2f(this.program.uniform['inverseSpriteTextureSize'], this.spriteSheetScaleX, this.spriteSheetScaleY);
        gl.uniform2f(this.program.uniform['viewOffset'], Math.floor(x * scale), Math.floor(y * scale));
        gl.uniform2f(this.program.uniform['inverseTileTextureSize'], this.tileTextureScaleX, this.tileTextureScaleY);
        gl.uniform1f(this.program.uniform['tileSize'], this.tileSize);
        gl.uniform1f(this.program.uniform['inverseTileSize'], 1 / this.tileSize);

        gl.activeTexture(gl.TEXTURE0);
        gl.uniform1i(this.program.uniform['sprites'], 0);
        gl.bindTexture(gl.TEXTURE_2D, this.spriteSheet);

        gl.activeTexture(gl.TEXTURE1);
        gl.uniform1i(this.program.uniform['tiles'], 1);
        gl.bindTexture(gl.TEXTURE_2D, this.tileTexture);

        gl.drawArrays(gl.TRIANGLES, 0, 6);
    }

    private static vertexShader = [
        "attribute vec2 position;",
        "attribute vec2 texture;",

        "varying vec2 pixelCoord;",
        "varying vec2 texCoord;",

        "uniform vec2 viewOffset;",
        "uniform vec2 viewportSize;",
        "uniform vec2 inverseTileTextureSize;",
        "uniform float inverseTileSize;",

        "void main(void) {",
        "   pixelCoord = (texture * viewportSize) + viewOffset;",
        "   texCoord = pixelCoord * inverseTileTextureSize * inverseTileSize;",
        "   gl_Position = vec4(position, 0.0, 1.0);",
        "}"
    ].join("\n");

    private static fragmentShader = [
        "precision highp float;",

        "varying vec2 pixelCoord;",
        "varying vec2 texCoord;",

        "uniform sampler2D tiles;",
        "uniform sampler2D sprites;",

        "uniform vec2 inverseTileTextureSize;",
        "uniform vec2 inverseSpriteTextureSize;",
        "uniform float tileSize;",

        "void main(void) {",
        "   vec4 tile = texture2D(tiles, texCoord);",
        "   if (texCoord.x < 0.0 || texCoord.x > 1.0 || texCoord.y < 0.0 || texCoord.y > 1.0 || (tile.x == 1.0 && tile.y == 1.0)) { discard; }",
        "   vec2 spriteOffset = floor(tile.xy * 256.0) * tileSize;",
        "   vec2 spriteCoord = mod(pixelCoord, tileSize);",
        "   gl_FragColor = texture2D(sprites, (spriteOffset + spriteCoord) * inverseSpriteTextureSize);",
        "}"
    ].join("\n");
}