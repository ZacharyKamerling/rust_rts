"use strict";
// Consumes binary data
var Cereal = (function () {
    function Cereal(dv) {
        this.offset = 0;
        this.dv = dv;
    }
    Cereal.prototype.get8 = function () {
        var val = this.dv.getInt8(this.offset);
        this.offset = this.offset + 1;
        return val;
    };
    Cereal.prototype.getU8 = function () {
        var val = this.dv.getUint8(this.offset);
        this.offset = this.offset + 1;
        return val;
    };
    Cereal.prototype.get16 = function () {
        var val = this.dv.getInt16(this.offset);
        this.offset = this.offset + 2;
        return val;
    };
    Cereal.prototype.getU16 = function () {
        var val = this.dv.getUint16(this.offset);
        this.offset = this.offset + 2;
        return val;
    };
    Cereal.prototype.get32 = function () {
        var val = this.dv.getInt32(this.offset);
        this.offset = this.offset + 4;
        return val;
    };
    Cereal.prototype.getU32 = function () {
        var val = this.dv.getUint32(this.offset);
        this.offset = this.offset + 4;
        return val;
    };
    Cereal.prototype.getF32 = function () {
        var val = this.dv.getFloat32(this.offset);
        this.offset = this.offset + 4;
        return val;
    };
    Cereal.prototype.getF64 = function () {
        var val = this.dv.getFloat64(this.offset);
        this.offset = this.offset + 8;
        return val;
    };
    Cereal.prototype.empty = function () {
        return (this.dv.byteLength === this.offset);
    };
    return Cereal;
})();
function uintToString(uintArray) {
    return decodeURIComponent(encodeURI(atob(String.fromCharCode.apply(null, uintArray))));
}
//# sourceMappingURL=cereal.js.map