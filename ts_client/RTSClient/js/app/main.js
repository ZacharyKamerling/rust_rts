var connectBtn = document.getElementById('connectBtn');
var connected = false;
var imageerLoaded = false;
var conn = null;
var game = new Game();
function playGame(conn, imageer) {
    if (!connected || !imageerLoaded) {
        return;
    }
    var mainMenu = document.getElementById('mainMenu');
    var content = document.getElementById('content');
    mainMenu.hidden = true;
    content.hidden = false;
    var actorCanvas = document.getElementById('actorCanvas');
    var tilemapCanvas = document.getElementById('tilemapCanvas');
    var fowCanvas = document.getElementById('fogOfWarCanvas');
    game.setActorCanvas(actorCanvas);
    game.setTilemapCanvas(tilemapCanvas);
    game.setFogOfWarCanvas(fowCanvas);
    game.setTilemap(new Tilemap(256, 256, "dirt0"));
    game.setImageer(imageer);
    game.setChef(new Chef());
    game.setConnection(conn);
    interact(fowCanvas, game.interactCanvas());
    var last_time = Date.now();
    function draw(time_passed) {
        var time_delta = (time_passed - last_time) / 100;
        console.log(time_delta);
        game.draw(time_delta);
        last_time = time_passed;
        requestAnimationFrame(draw);
    }
    draw(last_time);
}
function imageLoadData() {
    var imgs = [];
    imgs.push({ anim_count: 1, name: "dirt0", url: "../img/dirt0.png" });
    return imgs;
}
console.log('Script started...');
var imageer = new Imageer(imageLoadData(), function (imgr) {
    imageerLoaded = true;
    playGame(conn, imgr);
});
connectBtn.onclick = function () {
    var nameFieldValue = document.getElementById('nameField').value;
    var passFieldValue = document.getElementById('passField').value;
    var addrFieldValue = document.getElementById('addrField').value;
    var portFieldValue = document.getElementById('portField').value;
    console.log('Attempting connection...');
    conn = new WebSocket('ws://[' + addrFieldValue + ']:' + portFieldValue);
    var chef = new Chef();
    conn.binaryType = "arraybuffer";
    conn.onopen = function () {
        console.log('Connection open.');
        chef.putString(nameFieldValue);
        chef.putString(passFieldValue);
        conn.send(chef.done());
        connected = true;
        playGame(conn, imageer);
    };
    conn.onmessage = function (event) {
        var c = new Cereal(new DataView(event.data));
        console.log('Got data. ' + c.dv.byteLength);
        game.processPacket(c);
    };
    conn.onclose = function () {
        var mainMenu = document.getElementById('mainMenu');
        mainMenu.hidden = false;
        console.log('Connection closed.');
    };
};
//# sourceMappingURL=main.js.map