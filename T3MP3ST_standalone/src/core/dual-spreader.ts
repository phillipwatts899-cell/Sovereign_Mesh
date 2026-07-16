import * as crypto from 'crypto';

export interface DualProtocolPacket {
    sequence: number;
    mode: 'MIST_SHIFT' | 'CHAFF_SHARD';
    payload: string;   // Sealed Base64 array
    signature: string; // HMAC validation tag
}

export class DualProtocolEngine {
    private secretKey: Buffer;

    constructor(sharedSecret: string) {
        this.secretKey = crypto.createHash('sha256').update(sharedSecret).digest();
    }

    /**
     * NORMAL MODE: Standard uniform mist movement
     */
    public normalMistShift(plaintext: string, chaffRatio: number = 5): DualProtocolPacket[] {
        const stream: DualProtocolPacket[] = [];
        const base64Wheat = Buffer.from(plaintext, 'utf-8').toString('base64');
        const wheatMac = crypto.createHmac('sha256', this.secretKey).update(base64Wheat).digest('hex');

        stream.push({
            sequence: crypto.randomInt(1000, 9999),
            mode: 'MIST_SHIFT',
            payload: base64Wheat,
            signature: wheatMac
        });

        for (let i = 0; i < chaffRatio; i++) {
            const fakePayload = crypto.randomBytes(base64Wheat.length).toString('base64');
            const fakeMac = crypto.randomBytes(32).toString('hex');
            stream.push({
                sequence: crypto.randomInt(1000, 9999),
                mode: 'MIST_SHIFT',
                payload: fakePayload,
                signature: fakeMac
            });
        }
        return stream.sort(() => Math.random() - 0.5);
    }

    /**
     * EMERGENCY INTERCEPT MODE: Shatters the payload and hides pieces inside noise
     */
    public shatterAndSpread(plaintext: string, totalShards: number = 3, noiseDensity: number = 4): DualProtocolPacket[] {
        const base64Data = Buffer.from(plaintext, 'utf-8').toString('base64');
        const chunkSize = Math.ceil(base64Data.length / totalShards);
        const stream: DualProtocolPacket[] = [];

        for (let i = 0; i < totalShards; i++) {
            const start = i * chunkSize;
            const end = Math.min(start + chunkSize, base64Data.length);
            const rawShard = base64Data.slice(start, end);

            // Hide the real shard right in the center of pure random chaff noise bytes
            const noise = crypto.randomBytes(64);
            const blended = Buffer.concat([
                noise.subarray(0, 32),
                Buffer.from(rawShard, 'utf-8'),
                noise.subarray(32)
            ]);

            const containerBase64 = blended.toString('base64');
            const signature = crypto.createHmac('sha256', this.secretKey).update(containerBase64).digest('hex');

            stream.push({
                sequence: crypto.randomInt(10000, 99999),
                mode: 'CHAFF_SHARD',
                payload: containerBase64,
                signature: signature
            });

            // Inject standalone decoy noise arrays to completely distort the signal
            for (let c = 0; c < noiseDensity; c++) {
                const pureChaff = crypto.randomBytes(blended.length).toString('base64');
                const fakeMac = crypto.randomBytes(32).toString('hex');
                stream.push({
                    sequence: crypto.randomInt(10000, 99999),
                    mode: 'CHAFF_SHARD',
                    payload: pureChaff,
                    signature: fakeMac
                });
            }
        }
        return stream.sort(() => Math.random() - 0.5);
    }
}

