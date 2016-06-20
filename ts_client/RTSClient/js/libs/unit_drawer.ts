class UnitDrawer {
    private spriteMap: { [index: string]: { x: number, y: number, w: number, h: number } } = {};
    private canvas: HTMLCanvasElement;
    private gl: WebGLRenderingContext;
    private spriteAtlas: WebGLTexture;
    private spriteImage: HTMLImageElement;
    private buffer: WebGLBuffer;
    private program: MetaProgram;
    private atlasWidth: number = 0;
    private atlasHeight: number = 0;

    constructor(canvas: HTMLCanvasElement, spriteSrc: string, spriteMap: { [index: string]: { x: number, y: number, w: number, h: number } }) {
        this.canvas = canvas;
        let gl = <WebGLRenderingContext>this.canvas.getContext('webgl');
        // TODO See if this is even needed
        // gl.viewport(0, 0, canvas.width, canvas.height);
        this.program = new MetaProgram(gl, createProgram(gl, UnitDrawer.vertexShader, UnitDrawer.fragmentShader));
        this.spriteAtlas = gl.createTexture();
        this.spriteMap = spriteMap;

        let self = this;
        let sprts = new Image();

        sprts.onerror = function (e: Event) {
            console.log('Failed to load ' + spriteSrc);
        };

        sprts.onload = function (e: Event) {
            console.log('Loaded ' + spriteSrc);
            self.gl.bindTexture(self.gl.TEXTURE_2D, self.spriteAtlas);
            self.gl.texImage2D(self.gl.TEXTURE_2D, 0, self.gl.RGBA, self.gl.RGBA, self.gl.UNSIGNED_BYTE, sprts);
            self.gl.texParameteri(self.gl.TEXTURE_2D, self.gl.TEXTURE_MAG_FILTER, self.gl.NEAREST);
            self.gl.texParameteri(self.gl.TEXTURE_2D, self.gl.TEXTURE_MIN_FILTER, self.gl.NEAREST);
            this.atlasWidth = sprts.width;
            this.atlasHeight = sprts.height;
        };

        sprts.src = spriteSrc;

        this.buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([]), gl.STATIC_DRAW);

        /*
        this.canvas.style.position = "absolute";
        this.canvas.style.left = "0px";
        this.canvas.style.top = "0px";
        this.canvas.style.zIndex = "1";
        */
    }

    public draw(x: number, y: number, scale: number, units: [{ x: number, y: number, r: number, sprite: string }]) {
        /*
        let buffer = [
            //x  y  u  v
            -1, -1, 0, 1,
            1, -1, 1, 1,
            1, 1, 1, 0,

            -1, -1, 0, 1,
            1, 1, 1, 0,
            -1, 1, 0, 0
        ];
        */

        const BYTES_PER_UNIT = 24;
        let drawData = new Float32Array(BYTES_PER_UNIT * units.length);

        // The maximum supported value is 65536 which is the far right/top of the map.
        // Each tile is 16x16 and the map has a maximum size of 1024x1024.
        let xm = ((65536 / (16 * 1024)) / this.canvas.width);
        let ym = ((65536 / (16 * 1024)) / this.canvas.height);

        for (let i = 0, n = 0; n < units.length; n++) {
            let xywh = this.spriteMap[units[n].sprite];
            let hw = xywh.w / 2;
            let hh = xywh.h / 2;
            let qrtrPI = Math.PI / 4;
            let ascX = hw * Math.SQRT2 * Math.cos(units[i].r + qrtrPI);
            let ascY = hh * Math.SQRT2 * Math.sin(units[i].r + qrtrPI);
            let desX = hw * Math.SQRT2 * Math.cos(units[i].r - qrtrPI);
            let desY = hh * Math.SQRT2 * Math.sin(units[i].r - qrtrPI);
            
            // Negative & Positive coords scaled
            let neX = (units[i].x + ascX) * xm;
            let neY = (units[i].y + ascY) * ym;
            let swX = (units[i].x - ascX) * xm;
            let swY = (units[i].y - ascY) * ym;
            let nwX = (units[i].x - desX) * xm;
            let nwY = (units[i].y + desY) * ym;
            let seX = (units[i].x + desX) * xm;
            let seY = (units[i].y - desY) * ym;

            drawData[i + 0] = swX;
            drawData[i + 1] = swY;
            drawData[i + 2] = 0;
            drawData[i + 3] = 1;

            drawData[i + 4] = seX;
            drawData[i + 5] = seY;
            drawData[i + 6] = 1;
            drawData[i + 7] = 1;
            
            drawData[i + 8] = neX;
            drawData[i + 9] = neY;
            drawData[i + 10] = 1;
            drawData[i + 11] = 0;

            drawData[i + 12] = swX;
            drawData[i + 13] = swY;
            drawData[i + 14] = 0;
            drawData[i + 15] = 1;

            drawData[i + 16] = neX;
            drawData[i + 17] = neY;
            drawData[i + 18] = 1;
            drawData[i + 19] = 0;

            drawData[i + 20] = nwX;
            drawData[i + 21] = nwY;
            drawData[i + 22] = 0;
            drawData[i + 23] = 0;
            i += BYTES_PER_UNIT;
        }

        let gl = <WebGLRenderingContext>this.canvas.getContext('webgl');
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.useProgram(this.program.program);

        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
        gl.bufferData(gl.ARRAY_BUFFER, drawData, gl.STATIC_DRAW);

        gl.enableVertexAttribArray(this.program.attribute['a_position']);
        gl.enableVertexAttribArray(this.program.attribute['a_texture_coord']);
        gl.vertexAttribPointer(this.program.attribute['a_position'], 2, gl.FLOAT, false, 16, 0);
        gl.vertexAttribPointer(this.program.attribute['a_texture_coord'], 2, gl.FLOAT, false, 16, 8);

        gl.uniform2f(this.program.uniform['viewOffset'], x, y);
        gl.uniform2f(this.program.uniform['viewportSize'], this.canvas.width, this.canvas.height);

        gl.activeTexture(gl.TEXTURE0);
        gl.uniform1i(this.program.uniform['sprites'], 0);
        gl.bindTexture(gl.TEXTURE_2D, this.spriteAtlas);

        gl.drawArrays(gl.TRIANGLES, 0, 6 * units.length);
    }

    private static vertexShader = [
        "precision highp float;",

        "attribute vec2 a_position;",
        "attribute vec2 a_texture_coord;",

        "uniform vec2 viewOffset;",
        "uniform vec2 viewportSize;",

        "void main() {",
        "    gl_Position = vec4(a_position + viewOffset, 0.0, 1.0);",
        "    v_texture_coord = a_texture_coord;",
        "}",
    ].join("\n");

    private static fragmentShader = [
        "precision highp float;",

        "varying vec2 v_texture_coord;",

        "uniform sampler2D u_sampler;",

        "void main() {",
        "    gl_FragColor = texture2D(u_sampler, v_texture_coord);",
        "}",
    ].join("\n");
}