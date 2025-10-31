class WebSocketManager {
    constructor(wss) {
        this.wss = wss
        this.setupConnectionHandler()
    }

    setupConnectionHandler() {
        this.wss.on('connection', (ws) => {
            console.log('📱 Frontend connected to WebSocket')

            // When a new client connects, send them a welcome message
            ws.send(JSON.stringify({
                type: 'connected_to_server',
                message: 'WebSocket connection established'
            }))
        })
    }

    broadcast(data) {
        const message = JSON.stringify(data)
        let sentCount = 0
        for (const client of this.wss.clients) {
            if (client.readyState === 1) { // OPEN
                client.send(message)
                sentCount++
            }
        }
        if (sentCount > 0) {
            console.log(`📤 Broadcasted to ${sentCount} client(s)`)
        }
    }

    sendQr(qr) {
        console.log('📱 Sending QR code to frontend')
        this.broadcast({ type: "qr", qr })
    }

    sendConnected() {
        console.log('✅ Sending connected status to frontend')
        this.broadcast({ type: "connected" })
    }

    sendMessage(message) {
        this.broadcast({ type: "message", message })
    }

    sendContacts(contacts) {
        console.log(`📇 [WebSocketManager] Sending ${contacts.length} contacts to frontend`)
        if (contacts.length > 0) {
            console.log(`📇 Sample contacts being sent:`, JSON.stringify(contacts.slice(0, 3).map(c => ({
                jid: c.jid,
                name: c.name,
                isGroup: c.isGroup
            })), null, 2))
        }
        this.broadcast({ type: "contacts", contacts })
    }
}

export default WebSocketManager
