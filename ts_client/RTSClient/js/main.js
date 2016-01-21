define(["require", "exports", "libs/imageer", "libs/chef", "game"], function (require, exports, IMG, C, G) {
    window.onload = function () {
        var connectBtn = document.getElementById('connectBtn');
        var connected = false;
        var imageerLoaded = false;
        var conn = null;
        var imageer = new IMG.Imageer(imageLoadData(), function (imgr) {
            imageerLoaded = true;
            playGame(connected, imageerLoaded, conn, imgr);
        });
        connectBtn.onclick = function () {
            var nameFieldValue = document.getElementById('nameField').value;
            var passFieldValue = document.getElementById('passField').value;
            var addrFieldValue = document.getElementById('addrField').value;
            var portFieldValue = document.getElementById('portField').value;
            console.log('Attempting connection...');
            conn = new WebSocket('ws://[' + addrFieldValue + ']:' + portFieldValue);
            var chef = new C.Chef();
            conn.binaryType = "arraybuffer";
            conn.onopen = function () {
                console.log('Connection open.');
                chef.putString(nameFieldValue);
                chef.putString(passFieldValue);
                conn.send(chef.done());
                connected = true;
                playGame(connected, imageerLoaded, conn, imageer);
            };
            conn.onmessage = function (event) {
                new Cereal(new DataView(event.data));
                console.log('Got data.');
            };
            conn.onclose = function () {
                var mainMenu = document.getElementById('mainMenu');
                mainMenu.hidden = false;
                console.log('Connection closed.');
            };
        };
    };
    function playGame(connected, imageerLoaded, conn, imageer) {
        if (!connected || !imageerLoaded) {
            return;
        }
        var mainMenu = document.getElementById('mainMenu');
        mainMenu.hidden = true;
        var content = document.getElementById('content');
        content.hidden = false;
        var game = new G.Game();
    }
    function imageLoadData() {
        var imgs = [];
        imgs.push({ anim_count: 1, name: "dirt0", url: "../img/dirt0.png" });
        return imgs;
    }
});
//# sourceMappingURL=main.js.map