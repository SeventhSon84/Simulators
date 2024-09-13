// src/main.js

//import { invoke } from '@tauri-apps/api'

const { invoke } = window.__TAURI__.tauri

document.addEventListener('DOMContentLoaded', () => {
    // Fetch the js_port from the Rust backend
    invoke('get_js_port')
        .then(jsPort => {
            const ws = new WebSocket(`ws://127.0.0.1:${jsPort}`);
            
            const statusElement = document.getElementById('status');
            const valueInput = document.getElementById('valueInput');
            const sendButton = document.getElementById('sendButton');
            const errorButton = document.getElementById('errorButton');

            ws.onmessage = (event) => {
                const data = JSON.parse(event.data);
                console.log('Received message:', data); // Add logging for debugging

                if (data.event === 'statusChange') {
                    const status = data.status;
                    statusElement.textContent = status;

                    // Update button state and color based on status
                    if (status === 'ERROR') {
                        sendButton.disabled = true;
                        errorButton.style.backgroundColor = 'red';
                        errorButton.style.color = 'white';
                    } else if (status === 'ARMED') {
                        sendButton.disabled = false;
                        errorButton.style.backgroundColor = ''; // Reset to default color
                        errorButton.style.color = ''; // Reset to default color
                    } else if (status === 'DISABLED') {
                        sendButton.disabled = true;
                        errorButton.style.backgroundColor = ''; // Reset to default color
                        errorButton.style.color = ''; // Reset to default color
                    }
                } else if (data.event === 'read') {
                    console.log('Value read:', data.value);
                }
            };

            sendButton.addEventListener('click', () => {
                const value = valueInput.value;
                ws.send(JSON.stringify({
                    action: 'read',
                    value
                }));
            });

            errorButton.addEventListener('click', () => {
                const status = statusElement.textContent;
                const newStatus = status === 'ERROR' ? 'DISABLED' : 'ERROR';
                ws.send(JSON.stringify({
                    action: 'error',
                    status: newStatus
                }));
            });
        })
        .catch(error => console.error('Error fetching js_port:', error));
});
