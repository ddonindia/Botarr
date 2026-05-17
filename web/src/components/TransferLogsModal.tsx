import React, { useEffect, useState, useRef } from 'react';
import { X, Terminal } from 'lucide-react';

interface TransferLogsModalProps {
    transferId: string;
    fileName?: string;
    bot?: string;
    onClose: () => void;
}

export const TransferLogsModal: React.FC<TransferLogsModalProps> = ({ transferId, fileName, bot, onClose }) => {
    const [logs, setLogs] = useState<string[]>([]);
    const [loading, setLoading] = useState(true);
    const endOfLogsRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const fetchLogs = async () => {
            try {
                const response = await fetch(`/api/transfers/${transferId}/logs`);
                if (response.ok) {
                    const data = await response.json();
                    setLogs(data.logs || []);
                }
            } catch (err) {
                console.error("Failed to fetch logs:", err);
            } finally {
                setLoading(false);
            }
        };

        fetchLogs();
        const interval = setInterval(fetchLogs, 2000);
        return () => clearInterval(interval);
    }, [transferId]);

    useEffect(() => {
        endOfLogsRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [logs]);

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-surface border border-white/10 rounded-xl shadow-2xl w-full max-w-4xl max-h-[80vh] flex flex-col overflow-hidden">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-white/5 bg-white/5">
                    <div className="flex items-center gap-3">
                        <Terminal size={20} className="text-primary" />
                        <div>
                            <h3 className="font-semibold text-white">Transfer Logs</h3>
                            <p className="text-xs text-muted truncate max-w-xl">
                                {fileName || `Bot: ${bot}` || transferId}
                            </p>
                        </div>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 rounded-lg hover:bg-white/10 text-secondary hover:text-white transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                {/* Logs Area */}
                <div className="flex-1 overflow-y-auto p-4 bg-[#0a0a0a] font-mono text-sm leading-relaxed">
                    {loading && logs.length === 0 ? (
                        <div className="flex items-center justify-center h-full text-muted animate-pulse">
                            Loading logs...
                        </div>
                    ) : logs.length === 0 ? (
                        <div className="flex items-center justify-center h-full text-muted">
                            No logs available yet.
                        </div>
                    ) : (
                        <div className="space-y-1">
                            {logs.map((log, index) => {
                                // Basic highlighting for common patterns
                                const isError = log.toLowerCase().includes('error') || log.toLowerCase().includes('fail');
                                const isSuccess = log.toLowerCase().includes('success') || log.toLowerCase().includes('completed');
                                const isNotice = log.toLowerCase().includes('notice from');
                                
                                let colorClass = 'text-gray-300';
                                if (isError) colorClass = 'text-red-400';
                                else if (isSuccess) colorClass = 'text-green-400';
                                else if (isNotice) colorClass = 'text-blue-300';

                                return (
                                    <div key={index} className={`break-words ${colorClass}`}>
                                        {log}
                                    </div>
                                );
                            })}
                            <div ref={endOfLogsRef} />
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};
