class WebSocketManager {
    constructor(wss) {
        this.wss = wss
        this.setupConnectionHandler()
    }

    setupConnectionHandler() {
        this.wss.on('connection', (ws) => {
            console.log('Frontend connected to WebSocket')

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
        if (sentCount > 0 && data.type !== 'presence.update') { // Avoid logging presence spam
            console.log(`Broadcasted event '${data.type}' to ${sentCount} client(s)`)
        }
    }

    sendEvent(type, payload) {
        this.broadcast({ type, payload })
    }
}

export default WebSocketManager
