import React from 'react';
import { XdccTransfer } from '../types';
import { formatSpeed } from '../utils/format';
import { X, RefreshCw } from 'lucide-react';

interface TransferListProps {
    transfers: XdccTransfer[];
    onCancel: (id: string) => void;
    onRetry: (id: string) => void;
}

export const TransferList: React.FC<TransferListProps & { onRefresh?: () => void }> = ({ transfers, onCancel, onRetry, onRefresh }) => {
    return (
        <div className="space-y-4">
            <div className="flex justify-between items-center mb-4">
                <h2 className="text-xl font-bold text-white">Active Transfers</h2>
                {onRefresh && (
                    <button
                        onClick={onRefresh}
                        className="p-2 hover:bg-white/10 rounded-lg text-secondary hover:text-white transition-colors"
                        title="Refresh Status"
                    >
                        <RefreshCw size={18} />
                    </button>
                )}
            </div>

            {transfers.length === 0 ? (
                <div className="text-center py-12 glass rounded-xl border-dashed border-2 border-white/10">
                    <p className="text-muted">No active transfers</p>
                </div>
            ) : (
                transfers.map((transfer) => (
                    <div key={transfer.id} className="glass-card p-4 flex items-center justify-between group">
                        <div className="flex-1 min-w-0 pr-4">
                            <div className="flex items-center gap-2 mb-1">
                                <span className={`w-2 h-2 rounded-full ${getStatusColor(transfer.status)}`} />
                                <h4 className="text-white font-medium truncate" title={transfer.file_name || 'Connecting...'}>
                                    {transfer.file_name || `Connecting to ${transfer.url.bot}...`}
                                </h4>
                            </div>
                            <div className="flex items-center gap-4 text-xs text-secondary mt-2">
                                <span className="bg-surface px-2 py-0.5 rounded text-muted border border-white/5">
                                    {transfer.url.server}
                                </span>
                                <span className="bg-surface px-2 py-0.5 rounded text-muted border border-white/5">
                                    {transfer.url.bot} #{transfer.url.pack}
                                </span>
                                {transfer.status === 'downloading' && (
                                    <>
                                        <span>{formatSpeed(transfer.speed)}</span>
                                        <span>{Math.round(transfer.progress)}%</span>
                                    </>
                                )}
                                {(['pending', 'connecting', 'joining', 'requesting'].includes(transfer.status)) && (
                                    <span className="text-primary animate-pulse">{transfer.status}...</span>
                                )}
                            </div>
                        </div>

                        <div className="flex items-center gap-3">
                            <div className="w-32 h-2 bg-surface rounded-full overflow-hidden">
                                {transfer.status === 'downloading' ? (
                                    <div
                                        className="h-full bg-primary transition-all duration-500 ease-out"
                                        style={{ width: `${transfer.progress}%` }}
                                    />
                                ) : (['pending', 'connecting', 'joining', 'requesting'].includes(transfer.status)) && (
                                    <div className="h-full bg-primary/30 w-full animate-pulse" />
                                )}
                            </div>

                            {(transfer.status === 'failed' || transfer.status === 'cancelled') ? (
                                <button
                                    onClick={() => onRetry(transfer.id)}
                                    className="p-2 hover:bg-white/10 rounded-lg text-secondary hover:text-white transition-colors"
                                    title="Retry"
                                >
                                    <RefreshCw size={18} />
                                </button>
                            ) : (
                                <button
                                    onClick={() => onCancel(transfer.id)}
                                    className="p-2 hover:bg-error/20 rounded-lg text-secondary hover:text-error transition-colors"
                                    title="Cancel"
                                >
                                    <X size={18} />
                                </button>
                            )}
                        </div>
                    </div>
                ))
            )}
        </div>
    );
};

function getStatusColor(status: string) {
    switch (status) {
        case 'downloading': return 'bg-success animate-pulse';
        case 'completed': return 'bg-info';
        case 'failed': return 'bg-error';
        case 'cancelled': return 'bg-secondary';
        default: return 'bg-warning';
    }
}
