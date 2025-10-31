#!/usr/bin/env bash

# Start WhatsApp Linux Client
# This script starts both the backend and frontend

echo "Starting WhatsApp Linux Client..."

# Check if backend dependencies are installed
if [ ! -d "baileys-backend/node_modules" ]; then
    echo "Installing backend dependencies..."
    cd baileys-backend
    npm install
    cd ..
fi

# Create db directory if it doesn't exist
mkdir -p db

# Start backend
echo "Starting backend on port 3000 and WebSocket on port 8787..."
cd baileys-backend
node server.js &
BACKEND_PID=$!
cd ..

# Wait for backend to start
sleep 2

# Start frontend
echo "Starting frontend..."
cd whatsapp-frontend
cargo run --release

# Cleanup on exit
kill $BACKEND_PID
