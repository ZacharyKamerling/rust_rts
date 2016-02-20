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
    game.setTilemap(new Tilemap(256, 256, "dirt0"));
    for (var y = 16; y < 49; y++) {
        for (var x = 16; x < 49; x++) {
            game.tilemap.setTile(x, y * 3, "wall0");
        }
    }
    game.tilemap.setTile(32, 47, "wall0");
    game.setImageer(imageer);
    game.setChef(new Chef());
    game.setConnection(conn);
    interact(fowCanvas, game.interact_canvas());
    var last_time = Date.now();
    function draw(time_passed) {
        var time_delta = (time_passed - last_time) / 100;
        game.draw(time_delta);
        last_time = time_passed;
        requestAnimationFrame(draw);
    }
    draw(last_time);
}
function imageLoadData() {
    var imgs = [];
    imgs.push({ anim_count: 1, name: "dirt0", url: "../img/dirt0.png" });
    imgs.push({ anim_count: 1, name: "wall0", url: "../img/wall0.png" });
    imgs.push({ anim_count: 1, name: "b_unit", url: "../img/basic_unit.png" });
    imgs.push({ anim_count: 1, name: "b_misl", url: "../img/basic_missile.png" });
    imgs.push({ anim_count: 1, name: "b_wpn", url: "../img/basic_wpn.png" });
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
        conn.onmessage = function (event) {
            var c = new Cereal(new DataView(event.data));
            //console.log(c.dv.byteLength);
            game.processPacket(c);
        };
        conn.onclose = function () {
            var mainMenu = document.getElementById('mainMenu');
            mainMenu.hidden = false;
            console.log('Connection closed.');
            game.disconnected();
            connected = false;
        };
        console.log('Connection open.');
        chef.putString(nameFieldValue);
        chef.putString(passFieldValue);
        conn.send(chef.done());
        connected = true;
        playGame(conn, imageer);
    };
};
//# sourceMappingURL=main.js.map