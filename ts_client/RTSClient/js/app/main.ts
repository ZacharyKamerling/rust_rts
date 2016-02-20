let connectBtn = document.getElementById('connectBtn');
let connected = false;
let imageerLoaded = false;
let conn: WebSocket = null;
let game = new Game();

function playGame(conn: WebSocket, imageer: Imageer) {
    if (!connected || !imageerLoaded) {
        return;
    }

    let mainMenu = document.getElementById('mainMenu');
    let content = document.getElementById('content');
    mainMenu.hidden = true;
    content.hidden = false;

    let actorCanvas = <HTMLCanvasElement>document.getElementById('actorCanvas');
    let tilemapCanvas = <HTMLCanvasElement>document.getElementById('tilemapCanvas');
    let fowCanvas = <HTMLCanvasElement>document.getElementById('fogOfWarCanvas');

    game.setActorCanvas(actorCanvas);
    game.setTilemapCanvas(tilemapCanvas);
    game.setTilemap(new Tilemap(256, 256, "dirt0"));
    
    for (let y = 16; y < 49; y++) {
        for (let x = 16; x < 49; x++) {
            game.tilemap.setTile(x, y * 3, "wall0");
        }
    }

    game.tilemap.setTile(32, 47, "wall0");

    game.setImageer(imageer);
    game.setChef(new Chef());
    game.setConnection(conn);
    interact(fowCanvas, game.interact_canvas());

    let last_time = Date.now();

    function draw(time_passed: number) {
        let time_delta = (time_passed - last_time) / 100;
        game.draw(time_delta);
        last_time = time_passed;
        requestAnimationFrame(draw);
    }

    draw(last_time);
}

function imageLoadData() {
    let imgs: { anim_count: number; name: string; url: string }[] = [];
    imgs.push({ anim_count: 1, name: "dirt0", url: "../img/dirt0.png" });
    imgs.push({ anim_count: 1, name: "wall0", url: "../img/wall0.png" });
    imgs.push({ anim_count: 1, name: "b_unit", url: "../img/basic_unit.png" });
    imgs.push({ anim_count: 1, name: "b_misl", url: "../img/basic_missile.png" });
    imgs.push({ anim_count: 1, name: "b_wpn", url: "../img/basic_wpn.png" });
    return imgs;
}

console.log('Script started...');
let imageer = new Imageer(imageLoadData(), function (imgr) {
    imageerLoaded = true;
    playGame(conn, imgr);
});

connectBtn.onclick = function () {
    let nameFieldValue = (<HTMLInputElement>document.getElementById('nameField')).value;
    let passFieldValue = (<HTMLInputElement>document.getElementById('passField')).value;
    let addrFieldValue = (<HTMLInputElement>document.getElementById('addrField')).value;
    let portFieldValue = (<HTMLInputElement>document.getElementById('portField')).value;
    console.log('Attempting connection...');
    conn = new WebSocket('ws://[' + addrFieldValue + ']:' + portFieldValue);
    let chef = new Chef();

    conn.binaryType = "arraybuffer";

    conn.onopen = function () {

        conn.onmessage = function (event) {
            let c = new Cereal(new DataView(event.data));
            //console.log(c.dv.byteLength);
            game.processPacket(c);
        }

        conn.onclose = function () {
            let mainMenu = document.getElementById('mainMenu');
            mainMenu.hidden = false;
            console.log('Connection closed.');
            game.disconnected();
            connected = false;
        }

        console.log('Connection open.');
        chef.putString(nameFieldValue);
        chef.putString(passFieldValue);
        conn.send(chef.done());
        connected = true;
        playGame(conn, imageer);
    }
};