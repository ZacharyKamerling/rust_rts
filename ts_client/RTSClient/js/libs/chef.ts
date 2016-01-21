declare var StringView;

// Cooks up binary data
class Chef {
    private ab = new ArrayBuffer(100);
    private dv = new DataView(this.ab);
    private offset = 0;

    resize(spaceNeeded): void {
        if (this.ab.byteLength < this.offset + spaceNeeded) {
            var newAB = new ArrayBuffer((this.ab.byteLength + spaceNeeded) * 2);
            var newDV = new DataView(newAB);
            for (var i = 0; i < this.offset; i++) {
                newDV.setInt8(i,this.dv.getInt8(i));
            }
            this.dv = newDV;
            this.ab = newAB;
        }
    }

    // Trim empty space and get array buffer
    done(): ArrayBuffer {
        var newAB = new ArrayBuffer(this.offset);
        var newDV = new DataView(newAB);
        for (var i = 0; i < this.offset; i++) {
            newDV.setInt8(i,this.dv.getInt8(i));
        }
        this.offset = 0;
        return newAB;
    }
    /*
    putString(str: string): void {
        var sv = StringView(str);
        this.resize(sv.buffer.byteLength + 2);
        this.dv.setUint16(this.offset,sv.buffer.byteLength);
        this.offset = this.offset + 2;
        for (var i = 0; i < sv.buffer.byteLength; i++) {
            this.putU8(sv.rawData[i]);
        }
    }
    */
    putString(str: string): void {
        var strBuff = this.toUTF8Array(str);
        this.resize(strBuff.length + 2);
        this.dv.setUint16(this.offset, strBuff.length);
        this.offset = this.offset + 2;
        for (var i = 0; i < strBuff.length; i++) {
            this.putU8(strBuff[i]);
        }
    }

    put8(v: number): void {
        this.resize(1);
        this.dv.setInt8(this.offset,v);
        this.offset = this.offset + 1;
    }

    putU8(v: number): void {
        this.resize(1);
        this.dv.setUint8(this.offset,v);
        this.offset = this.offset + 1;
    }

    put16(v: number): void {
        this.resize(2);
        this.dv.setInt16(this.offset,v);
        this.offset = this.offset + 2;
    }

    putU16(v: number): void {
        this.resize(2);
        this.dv.setUint16(this.offset,v);
        this.offset = this.offset + 2;
    }

    putU32(v: number): void {
        this.resize(4);
        this.dv.setUint32(this.offset,v);
        this.offset = this.offset + 4;
    }

    put32(v: number): void {
        this.resize(4);
        this.dv.setInt32(this.offset,v);
        this.offset = this.offset + 4;
    }

    putF32(v: number): void {
        this.resize(4);
        this.dv.setFloat32(this.offset,v);
        this.offset = this.offset + 4;
    }

    putF64(v: number): void {
        this.resize(8);
        this.dv.setFloat64(this.offset,v);
        this.offset = this.offset + 8;
    }

    private toUTF8Array(str: string): number[] {
        var utf8 = [];
        for (var i = 0; i < str.length; i++) {
            var charcode = str.charCodeAt(i);
            if (charcode < 0x80) utf8.push(charcode);
            else if (charcode < 0x800) {
                utf8.push(0xc0 | (charcode >> 6),
                    0x80 | (charcode & 0x3f));
            }
            else if (charcode < 0xd800 || charcode >= 0xe000) {
                utf8.push(0xe0 | (charcode >> 12),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            }
            // surrogate pair
            else {
                i++;
                // UTF-16 encodes 0x10000-0x10FFFF by
                // subtracting 0x10000 and splitting the
                // 20 bits of 0x0-0xFFFFF into two halves
                charcode = 0x10000 + (((charcode & 0x3ff) << 10)
                    | (str.charCodeAt(i) & 0x3ff));
                utf8.push(0xf0 | (charcode >> 18),
                    0x80 | ((charcode >> 12) & 0x3f),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            }
        }
        return utf8;
    }
}