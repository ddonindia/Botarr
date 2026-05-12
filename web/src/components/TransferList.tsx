import React from 'react';
import { XdccTransfer } from '../types';
import { formatSpeed } from '../utils/format';
import { X, RefreshCw, Trash2, Play, Terminal } from 'lucide-react';
import { TransferLogsModal } from './TransferLogsModal';

const FINISHED_STATUSES = ['completed', 'failed', 'cancelled'];
const ACTIVE_STATUSES = ['pending', 'connecting', 'joining', 'requesting', 'downloading', 'paused'];

interface TransferListProps {
    transfers: XdccTransfer[];
    onCancel: (id: string) => void;
    onRetry: (id: string) => void;
    onResume: (id: string) => void;
    onDelete: (id: string) => void;
    onClearFinished?: () => void;
}

export const TransferList: React.FC<TransferListProps & { onRefresh?: () => void }> = ({ transfers, onCancel, onRetry, onResume, onDelete, onClearFinished, onRefresh }) => {
    const hasFinished = transfers.some(t => FINISHED_STATUSES.includes(t.status));
    const [selectedTransfer, setSelectedTransfer] = React.useState<XdccTransfer | null>(null);
    const [filter, setFilter] = React.useState<'all' | 'active' | 'pending' | 'paused'>('all');

    const filteredTransfers = React.useMemo(() => {
        let result = transfers;
        if (filter === 'active') {
            result = transfers.filter(t => ['downloading', 'connecting', 'joining', 'requesting'].includes(t.status));
        } else if (filter === 'pending') {
            result = transfers.filter(t => t.status === 'pending');
        } else if (filter === 'paused') {
            result = transfers.filter(t => t.status === 'paused');
        }

        // Sort: active first, then paused, then finished
        return result.sort((a, b) => {
            const aIsPaused = a.status === 'paused';
            const bIsPaused = b.status === 'paused';
            if (aIsPaused && !bIsPaused) return 1;
            if (!aIsPaused && bIsPaused) return -1;
            return 0;
        });
    }, [transfers, filter]);

    return (
        <div className="space-y-4">
            <div className="flex justify-between items-center mb-4">
                <div className="flex flex-col sm:flex-row sm:items-center gap-4">
                    <h2 className="text-xl font-bold text-white">Transfers</h2>
                    <div className="flex bg-white/5 rounded-lg p-1">
                        <button onClick={() => setFilter('all')} className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${filter === 'all' ? 'bg-white/10 text-white' : 'text-secondary hover:text-white'}`}>All</button>
                        <button onClick={() => setFilter('active')} className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${filter === 'active' ? 'bg-white/10 text-white' : 'text-secondary hover:text-white'}`}>Active</button>
                        <button onClick={() => setFilter('pending')} className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${filter === 'pending' ? 'bg-white/10 text-white' : 'text-secondary hover:text-white'}`}>Pending</button>
                        <button onClick={() => setFilter('paused')} className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${filter === 'paused' ? 'bg-white/10 text-white' : 'text-secondary hover:text-white'}`}>Paused</button>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    {hasFinished && onClearFinished && (
                        <button
                            onClick={onClearFinished}
                            className="px-3 py-1.5 text-xs font-medium hover:bg-error/20 rounded-lg text-secondary hover:text-error transition-colors flex items-center gap-1.5"
                            title="Clear all finished transfers"
                        >
                            <Trash2 size={14} />
                            Clear Finished
                        </button>
                    )}
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
            </div>

            {filteredTransfers.length === 0 ? (
                <div className="text-center py-12 glass rounded-xl border-dashed border-2 border-white/10">
                    <p className="text-muted">No {filter !== 'all' ? filter : ''} transfers</p>
                </div>
            ) : (
                filteredTransfers.map((transfer) => (
                    <div key={transfer.id} className="glass-card p-4 flex items-center justify-between group">
                        <div 
                            className="flex-1 min-w-0 pr-4 cursor-pointer" 
                            onClick={() => setSelectedTransfer(transfer)}
                        >
                            <div className="flex items-center gap-2 mb-1">
                                <span className={`w-2 h-2 rounded-full ${getStatusColor(transfer.status)}`} />
                                <h4 className="text-white font-medium truncate hover:text-primary transition-colors flex items-center gap-2" title={transfer.file_name || 'Connecting...'}>
                                    {transfer.file_name || `Connecting to ${transfer.url.bot}...`}
                                    <Terminal size={14} className="text-secondary opacity-50" />
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
                                {ACTIVE_STATUSES.includes(transfer.status) && transfer.status !== 'downloading' && (
                                    <span className="text-primary font-semibold uppercase tracking-wider animate-pulse bg-primary/10 px-2 py-0.5 rounded">
                                        {transfer.status}
                                    </span>
                                )}
                                {transfer.status === 'completed' && (
                                    <span className="text-info font-semibold uppercase tracking-wider bg-info/10 px-2 py-0.5 rounded">
                                        completed
                                    </span>
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
                                ) : transfer.status === 'completed' ? (
                                    <div className="h-full bg-info w-full" />
                                ) : transfer.status === 'paused' ? (
                                    <div className="h-full bg-secondary/50 w-full" />
                                ) : ACTIVE_STATUSES.includes(transfer.status) && (
                                    <div className="h-full bg-primary/30 w-full animate-pulse" />
                                )}
                            </div>

                            {FINISHED_STATUSES.includes(transfer.status) ? (
                                <div className="flex items-center gap-1">
                                    {(transfer.status === 'failed' || transfer.status === 'cancelled') && (
                                        <button
                                            onClick={() => onRetry(transfer.id)}
                                            className="p-2 hover:bg-white/10 rounded-lg text-secondary hover:text-white transition-colors"
                                            title="Retry"
                                        >
                                            <RefreshCw size={18} />
                                        </button>
                                    )}
                                    <button
                                        onClick={() => onDelete(transfer.id)}
                                        className="p-2 hover:bg-error/20 rounded-lg text-secondary hover:text-error transition-colors"
                                        title="Remove"
                                    >
                                        <X size={18} />
                                    </button>
                                </div>
                            ) : transfer.status === 'paused' ? (
                                <div className="flex items-center gap-1">
                                    <button
                                        onClick={() => onResume(transfer.id)}
                                        className="p-2 hover:bg-success/20 rounded-lg text-secondary hover:text-success transition-colors"
                                        title="Start Transfer"
                                    >
                                        <Play size={18} fill="currentColor" />
                                    </button>
                                    <button
                                        onClick={() => onCancel(transfer.id)}
                                        className="p-2 hover:bg-error/20 rounded-lg text-secondary hover:text-error transition-colors"
                                        title="Cancel"
                                    >
                                        <X size={18} />
                                    </button>
                                </div>
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

            {selectedTransfer && (
                <TransferLogsModal
                    transferId={selectedTransfer.id}
                    fileName={selectedTransfer.file_name || undefined}
                    bot={selectedTransfer.url.bot}
                    onClose={() => setSelectedTransfer(null)}
                />
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
        case 'paused': return 'bg-yellow-500';
        default: return 'bg-warning';
    }
}
