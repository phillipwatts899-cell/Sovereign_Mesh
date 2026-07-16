import * as crypto from 'crypto';

export interface DaemonNode {
    id: string;
    peers: string[];
}

export class RamanujanMeshEngine {
    private mesh: Map<string, DaemonNode> = new Map();
    private degree: number;

    constructor(degree: number = 4) {
        this.degree = degree;
    }

    public buildExpanderMesh(daemonIds: string[]): void {
        const numNodes = daemonIds.length;
        if (numNodes <= this.degree) {
            return; 
        }

        for (const id of daemonIds) {
            this.mesh.set(id, { id, peers: [] });
        }

        for (let i = 0; i < numNodes; i++) {
            const currentNodeId = daemonIds[i];
            const current = this.mesh.get(currentNodeId)!;

            for (let j = 1; j <= this.degree / 2; j++) {
                const forwardPeerIndex = (i + j) % numNodes;
                const backwardPeerIndex = (i - j + numNodes) % numNodes;

                const forwardPeerId = daemonIds[forwardPeerIndex];
                const backwardPeerId = daemonIds[backwardPeerIndex];

                if (!current.peers.includes(forwardPeerId) && forwardPeerId !== currentNodeId) {
                    current.peers.push(forwardPeerId);
                }
                if (!current.peers.includes(backwardPeerId) && backwardPeerId !== currentNodeId) {
                    current.peers.push(backwardPeerId);
                }
            }
        }
    }

    public propagateMistPacket(startNodeId: string, packet: any, stepCount: number = 0): void {
        const node = this.mesh.get(startNodeId);
        if (!node) return;

        if (stepCount > Math.log2(this.mesh.size)) {
            return;
        }

        const nextPeerId = node.peers[crypto.randomInt(0, node.peers.length)];
        console.log(`[Mesh Trace] 🌬️ Mist packet hopped from [${startNodeId}] ──► [${nextPeerId}] (Step ${stepCount + 1})`);
        
        this.propagateMistPacket(nextPeerId, packet, stepCount + 1);
    }
}

