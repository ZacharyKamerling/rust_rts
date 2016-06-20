"use strict";

function createProgram(ctx: WebGLRenderingContext, vertShaderSrc: string, fragShaderSrc: string): WebGLProgram {
    let createShader =
        function (ctx: WebGLRenderingContext, shaderSrc: string, shaderType: number): WebGLShader {
            let shader = ctx.createShader(shaderType);

            ctx.shaderSource(shader, shaderSrc);
            ctx.compileShader(shader);

            if (!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
                console.error(ctx.getShaderInfoLog(shader));
                ctx.deleteShader(shader);
                return null;
            }

            return shader;
        };

    let program = ctx.createProgram();
    let vertShader = createShader(ctx, vertShaderSrc, ctx.VERTEX_SHADER);
    let fragShader = createShader(ctx, fragShaderSrc, ctx.FRAGMENT_SHADER);

    ctx.attachShader(program, vertShader);
    ctx.attachShader(program, fragShader);
    ctx.linkProgram(program);

    if (!ctx.getProgramParameter(program, ctx.LINK_STATUS)) {
        console.error("Program Link error:", ctx.getProgramInfoLog(program));
        ctx.deleteProgram(program);
        ctx.deleteShader(vertShader);
        ctx.deleteShader(fragShader);
        return null;
    }

    return program;
}

class MetaProgram {
    public program: WebGLProgram;
    public attribute: { [index: string]: number };
    public uniform: { [index: string]: WebGLUniformLocation };

    constructor(gl: WebGLRenderingContext, program: WebGLProgram) {
        this.program = program;
        this.attribute = {};
        this.uniform = {};
        let count: number = gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES);

        for (let i = 0; i < count; i++) {
            let attrib = gl.getActiveAttrib(program, i);
            this.attribute[attrib.name] = gl.getAttribLocation(program, attrib.name);
        }

        count = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
        for (let i = 0; i < count; i++) {
            let uniform = gl.getActiveUniform(program, i);
            let name = uniform.name.replace("[0]", "");
            this.uniform[name] = gl.getUniformLocation(program, name);
        }
    }
}