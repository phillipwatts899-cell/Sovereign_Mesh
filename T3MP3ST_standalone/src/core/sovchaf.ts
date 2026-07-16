import * as crypto from 'crypto';

export interface SovchafPacket {
    sequence: number;
    payload: string;   
    mac: string;       
}

export class Base64SovchafEngine {
    private secretKey: Buffer;

    constructor(sharedSecret: string) {
        this.secretKey = crypto.createHash('sha256').update(sharedSecret).digest();
    }

    public sealAndChafe(plaintext: string, otpKey: Buffer, chaffRatio: number = 8): SovchafPacket[] {
        const streamPackets: SovchafPacket[] = [];
        const base64SealedWheat = Buffer.from(plaintext, 'utf-8').toString('base64');
        const wheatBuffer = Buffer.from(base64SealedWheat, 'utf-8');

        if (wheatBuffer.length > otpKey.length) {
            throw new Error(`[Søvchaf] OTP Key must be at least ${wheatBuffer.length} bytes.`);
        }

        const wheatCiphertext = Buffer.alloc(wheatBuffer.length);
        for (let i = 0; i < wheatBuffer.length; i++) {
            wheatCiphertext[i] = wheatBuffer[i] ^ otpKey[i];
        }

        const finalWheatPayload = wheatCiphertext.toString('base64');
        const wheatMac = crypto.createHmac('sha256', this.secretKey).update(finalWheatPayload).digest('hex');

        streamPackets.push({
            sequence: crypto.randomInt(100, 999),
            payload: finalWheatPayload,
            mac: wheatMac
        });

        for (let i = 0; i < chaffRatio; i++) {
            const fakeBytes = crypto.randomBytes(wheatBuffer.length);
            const finalChaffPayload = fakeBytes.toString('base64');
            const fakeMac = crypto.randomBytes(32).toString('hex');

            streamPackets.push({
                sequence: crypto.randomInt(100, 999),
                payload: finalChaffPayload,
                mac: fakeMac
            });
        }

        return streamPackets.sort(() => Math.random() - 0.5);
    }

    public winnowAndUnseal(packets: SovchafPacket[], otpKey: Buffer): string {
        let authenticatedPayload = "";

        for (const packet of packets) {
            const calculatedMac = crypto.createHmac('sha256', this.secretKey).update(packet.payload).digest('hex');
            if (calculatedMac === packet.mac) {
                authenticatedPayload = packet.payload;
                break;
            }
        }

        if (!authenticatedPayload) {
            throw new Error("[Søvchaf] Execution Aborted: Data block signature mismatch.");
        }

        const ciphertextBuffer = Buffer.from(authenticatedPayload, 'base64');
        const decryptedBase64Buffer = Buffer.alloc(ciphertextBuffer.length);

        for (let i = 0; i < ciphertextBuffer.length; i++) {
            decryptedBase64Buffer[i] = ciphertextBuffer[i] ^ otpKey[i];
        }

        const base64String = decryptedBase64Buffer.toString('utf-8');
        return Buffer.from(base64String, 'base64').toString('utf-8');
    }
}

