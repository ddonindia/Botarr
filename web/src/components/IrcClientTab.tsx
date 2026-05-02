import React, { useState, useEffect, useRef } from 'react';

interface WsMessage {
    type: string;
    network: string;
    target?: string | null;
    message: string;
}

interface BufferMessage {
    timestamp: string;
    type: string;
    message: string;
}

export function IrcClientTab() {
    const [ws, setWs] = useState<WebSocket | null>(null);
    const [buffers, setBuffers] = useState<Record<string, BufferMessage[]>>({});
    const [activeBuffer, setActiveBuffer] = useState<string>('System');
    const [input, setInput] = useState('');
    const chatEndRef = useRef<HTMLDivElement>(null);

    const scrollToBottom = () => {
        chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    };

    useEffect(() => {
        scrollToBottom();
    }, [buffers, activeBuffer]);

    useEffect(() => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/irc/ws`;
        const socket = new WebSocket(wsUrl);

        socket.onopen = () => {
            appendMessage('System', 'success', 'Connected to Web IRC Backend.');
        };

        socket.onmessage = (event) => {
            try {
                const msg: WsMessage = JSON.parse(event.data);
                const bufferName = msg.target ? `${msg.network}:${msg.target}` : msg.network;
                appendMessage(bufferName, msg.type, msg.message);
            } catch (e) {
                console.error("Failed to parse WS message", e);
            }
        };

        socket.onclose = () => {
            appendMessage('System', 'error', 'Disconnected from backend.');
        };

        setWs(socket);

        return () => {
            socket.close();
        };
    }, []);

    const appendMessage = (buffer: string, type: string, message: string) => {
        const timestamp = new Date().toLocaleTimeString();
        setBuffers(prev => {
            const current = prev[buffer] || [];
            return {
                ...prev,
                [buffer]: [...current, { timestamp, type, message }].slice(-500) // Keep last 500
            };
        });
        
        // Auto-switch to new buffer if it's the first message and we are just on System
        setBuffers(prev => {
            if (Object.keys(prev).length === 2 && prev['System']) {
                // If we just got our first real buffer, maybe switch to it?
                // Left manual for now.
            }
            return prev;
        });
    };

    const handleCommand = (cmd: string) => {
        if (!ws || ws.readyState !== WebSocket.OPEN) {
            appendMessage(activeBuffer, 'error', 'Not connected to backend.');
            return;
        }

        if (cmd.startsWith('/connect ')) {
            const parts = cmd.split(' ');
            if (parts.length < 2) {
                appendMessage(activeBuffer, 'error', 'Usage: /connect <network_name> [host] [port] [ssl:true|false] [nick]');
                return;
            }
            
            // Set defaults based on the network name or generic defaults
            const network = parts[1];
            let host = parts[2];
            let port = parseInt(parts[3], 10);
            let ssl = parts[4] === 'true';
            let nick = parts[5] || `Botarr_${Math.random().toString(36).substring(2, 8)}`;

            // Auto-fill common networks if only network name is provided
            if (parts.length === 2) {
                if (network.toLowerCase().includes('scenep2p')) {
                    host = 'irc.scenep2p.net'; port = 6697; ssl = true;
                } else if (network.toLowerCase().includes('abjects')) {
                    host = 'irc.abjects.net'; port = 6697; ssl = true;
                } else if (network.toLowerCase().includes('libera')) {
                    host = 'irc.libera.chat'; port = 6697; ssl = true;
                } else {
                    host = network; // assume the network name IS the host
                    port = 6697;
                    ssl = true;
                }
            } else {
                host = host || network;
                port = isNaN(port) ? 6697 : port;
                ssl = parts[4] !== undefined ? ssl : true;
            }

            ws.send(JSON.stringify({
                action: 'connect',
                network, host, port, ssl, nick
            }));
            setActiveBuffer(network);
        } else if (cmd.startsWith('/join ')) {
            const channel = cmd.split(' ')[1];
            const network = activeBuffer.split(':')[0];
            if (network === 'System') {
                appendMessage(activeBuffer, 'error', 'Please select a network buffer first.');
                return;
            }
            ws.send(JSON.stringify({
                action: 'send',
                network,
                message: `JOIN ${channel}`
            }));
            setActiveBuffer(`${network}:${channel}`);
        } else if (cmd.startsWith('/part ')) {
            const channel = cmd.split(' ')[1];
            const network = activeBuffer.split(':')[0];
            ws.send(JSON.stringify({
                action: 'send',
                network,
                message: `PART ${channel}`
            }));
        } else if (cmd.startsWith('/disconnect ')) {
             const network = cmd.split(' ')[1] || activeBuffer.split(':')[0];
             ws.send(JSON.stringify({
                 action: 'disconnect',
                 network
             }));
        } else if (cmd.startsWith('/msg ')) {
            const parts = cmd.split(' ');
            const target = parts[1];
            const msg = parts.slice(2).join(' ');
            const network = activeBuffer.split(':')[0];
            ws.send(JSON.stringify({
                action: 'send',
                network,
                message: `PRIVMSG ${target} :${msg}`
            }));
            appendMessage(`${network}:${target}`, 'message', `-> *${target}* ${msg}`);
        } else {
            // Normal message
            if (activeBuffer === 'System' || !activeBuffer.includes(':')) {
                // If it's a raw command to the server
                const network = activeBuffer.split(':')[0];
                if (network !== 'System') {
                    ws.send(JSON.stringify({
                        action: 'send',
                        network,
                        message: cmd
                    }));
                } else {
                    appendMessage(activeBuffer, 'error', 'Cannot send raw messages to System buffer.');
                }
            } else {
                // It's a channel message
                const parts = activeBuffer.split(':');
                const network = parts[0];
                const target = parts[1];
                ws.send(JSON.stringify({
                    action: 'send',
                    network,
                    message: `PRIVMSG ${target} :${cmd}`
                }));
                appendMessage(activeBuffer, 'message', `<You> ${cmd}`);
            }
        }
    };

    const handleInput = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter' && input.trim()) {
            handleCommand(input.trim());
            setInput('');
        }
    };

    const allBufferNames = ['System', ...Object.keys(buffers).filter(b => b !== 'System')];

    return (
        <div className="flex h-[calc(100vh-120px)] bg-gray-900/50 rounded-xl border border-gray-800 overflow-hidden">
            {/* Sidebar */}
            <div className="w-64 bg-gray-900/80 border-r border-gray-800 flex flex-col">
                <div className="p-4 border-b border-gray-800">
                    <h2 className="text-white font-bold">Buffers</h2>
                </div>
                <div className="flex-1 overflow-y-auto py-2">
                    {allBufferNames.map(buf => (
                        <div
                            key={buf}
                            onClick={() => setActiveBuffer(buf)}
                            className={`px-4 py-2 cursor-pointer text-sm truncate transition-colors ${
                                activeBuffer === buf 
                                    ? 'bg-blue-600/30 text-blue-400 border-l-2 border-blue-500' 
                                    : 'text-gray-400 hover:bg-gray-800'
                            }`}
                        >
                            {buf}
                        </div>
                    ))}
                </div>
            </div>

            {/* Main Chat Area */}
            <div className="flex-1 flex flex-col min-w-0">
                <div className="p-4 border-b border-gray-800 bg-gray-900/80">
                    <h2 className="text-white font-bold">{activeBuffer}</h2>
                </div>
                
                {activeBuffer === 'System' && (
                    <div className="absolute top-4 right-4 flex gap-2">
                        <button onClick={() => handleCommand(`/connect SceneP2P irc.scenep2p.net 6697 true Botarr_${Math.random().toString(36).substring(2, 8)}`)} className="px-3 py-1 bg-gray-800 hover:bg-gray-700 text-xs text-gray-300 rounded border border-gray-700">Connect SceneP2P</button>
                        <button onClick={() => handleCommand(`/connect Abjects irc.abjects.net 6697 true Botarr_${Math.random().toString(36).substring(2, 8)}`)} className="px-3 py-1 bg-gray-800 hover:bg-gray-700 text-xs text-gray-300 rounded border border-gray-700">Connect Abjects</button>
                    </div>
                )}
                
                <div className="flex-1 overflow-y-auto p-4 font-mono text-sm space-y-1">
                    {(buffers[activeBuffer] || []).map((msg, i) => (
                        <div key={i} className="flex gap-3 hover:bg-gray-800/30 px-2 py-1 rounded">
                            <span className="text-gray-500 shrink-0">{msg.timestamp}</span>
                            <span className={`break-words ${
                                msg.type === 'error' ? 'text-red-400' :
                                msg.type === 'status' ? 'text-green-400' :
                                'text-gray-300'
                            }`}>
                                {msg.message}
                            </span>
                        </div>
                    ))}
                    <div ref={chatEndRef} />
                </div>

                {/* Input Area */}
                <div className="p-4 bg-gray-900/80 border-t border-gray-800">
                    <input
                        type="text"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        onKeyDown={handleInput}
                        placeholder={activeBuffer === 'System' ? 'Type /connect <network> [host] [port] [ssl] [nick]' : `Message ${activeBuffer}...`}
                        className="w-full bg-gray-800 text-white rounded px-4 py-2 focus:outline-none focus:ring-1 focus:ring-blue-500 border border-gray-700 font-mono text-sm"
                    />
                </div>
            </div>
        </div>
    );
}
