module Misc {
    export function convertCanvasToImage(canvas: HTMLCanvasElement): HTMLImageElement {
        let image = new Image();
        image.src = canvas.toDataURL("image/png");
        return image;
    }

    export function normalizeAngle(f: number): number {
        while (f > Math.PI * 2.0) {
            f -= Math.PI * 2.0;
        }
        while (f < 0.0) {
            f += Math.PI * 2.0;
        }
        return f;
    }

    export function angularDistance(a: number, b: number): number {
        let dists = Math.abs(a - b);

        if (dists > Math.PI) {
            return 2.0 * Math.PI - dists;
        }
        else {
            return dists;
        }
    }

    // Angle to turn, angle to turn towards, amount to turn
    export function turnTowards(a: number, b: number, turn: number): number {
        let dist = angularDistance(a, b);

        if (turn > dist) {
            return b;
        }
        else {
            if (a > b) {
                if (a - b > Math.PI) {
                    return normalizeAngle(a + turn);
                }
                else {
                    return normalizeAngle(a - turn);
                }
            }
            else {
                if (b - a > Math.PI) {
                    return normalizeAngle(a - turn);
                }
                else {
                    return normalizeAngle(a + turn);
                }
            }
        }
    }

}