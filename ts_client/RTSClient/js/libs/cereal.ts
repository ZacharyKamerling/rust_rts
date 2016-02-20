"use strict";

// Consumes binary data
class Cereal {
    dv: DataView;
    offset: number = 0;
    constructor(dv: DataView) {
        this.dv = dv;
    }
    get8(): number {
        let val = this.dv.getInt8(this.offset);
        this.offset = this.offset + 1;
        return val;
    }

    getU8(): number {
        let val = this.dv.getUint8(this.offset);
        this.offset = this.offset + 1;
        return val;
    }

    get16(): number {
        let val = this.dv.getInt16(this.offset);
        this.offset = this.offset + 2;
        return val;
    }

    getU16(): number {
   		let val = this.dv.getUint16(this.offset);
   		this.offset = this.offset + 2;
   		return val;
   }

    get32(): number {
   		let val = this.dv.getInt32(this.offset);
   		this.offset = this.offset + 4;
   		return val;
   }

    getU32(): number {
   		let val = this.dv.getUint32(this.offset);
   		this.offset = this.offset + 4;
   		return val;
   }

    getF32(): number {
   		let val = this.dv.getFloat32(this.offset);
   		this.offset = this.offset + 4;
   		return val;
   }

    getF64(): number {
   		let val = this.dv.getFloat64(this.offset);
   		this.offset = this.offset + 8;
   		return val;
    }

    empty(): boolean {
        return (this.dv.byteLength === this.offset)
    }
}

function uintToString(uintArray): string {
    return decodeURIComponent(encodeURI(atob(String.fromCharCode.apply(null, uintArray))));
}