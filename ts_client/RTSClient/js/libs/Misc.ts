function convertCanvasToImage(canvas: HTMLCanvasElement) {
    var image = new Image();
    image.src = canvas.toDataURL("image/png");
    return image;
}