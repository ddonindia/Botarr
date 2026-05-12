import React, { useState } from 'react';
import { DownloadHistoryItem } from '../../hooks/useHistory';
import { CheckSquare, Square, RefreshCw, X, Trash2, Terminal, ChevronLeft, ChevronRight } from 'lucide-react';
import { formatBytes } from '../../utils/format';
import { TransferLogsModal } from '../TransferLogsModal';

interface DownloadHistoryProps {
    downloads: DownloadHistoryItem[];
    downloadPage: number;
    setDownloadPage: React.Dispatch<React.SetStateAction<number>>;
    downloadTotalPages: number;
    downloadTotal: number;
    fetchDownloadHistory: () => Promise<void>;
    deleteDownload: (id: string, deleteFile: boolean) => Promise<void>;
    bulkDeleteDownloads: (selectedIds: Set<string>, deleteFiles: boolean) => Promise<void>;
    clearAllDownloads: () => Promise<void>;
    loading: boolean;
}

export const DownloadHistory: React.FC<DownloadHistoryProps> = ({
    downloads,
    downloadPage,
    setDownloadPage,
    downloadTotalPages,
    downloadTotal,
    fetchDownloadHistory,
    deleteDownload,
    bulkDeleteDownloads,
    clearAllDownloads,
    loading
}) => {
    const [selectedDownloads, setSelectedDownloads] = useState<Set<string>>(new Set());
    const [selectedDownloadLog, setSelectedDownloadLog] = useState<DownloadHistoryItem | null>(null);

    const toggleDownloadSelection = (id: string) => {
        setSelectedDownloads(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    const selectAllDownloads = () => {
        if (selectedDownloads.size === downloads.length) {
            setSelectedDownloads(new Set());
        } else {
            setSelectedDownloads(new Set(downloads.map(d => d.id)));
        }
    };

    const handleBulkDelete = async (deleteFiles: boolean) => {
        if (selectedDownloads.size === 0) return;
        const msg = deleteFiles
            ? `Delete ${selectedDownloads.size} items AND their files from disk?`
            : `Remove ${selectedDownloads.size} items from history?`;
        if (!window.confirm(msg)) return;

        await bulkDeleteDownloads(selectedDownloads, deleteFiles);
        setSelectedDownloads(new Set());
    };

    const handleClearAll = async () => {
        if (!window.confirm(`Are you sure you want to clear ALL download history?`)) return;
        await clearAllDownloads();
        setSelectedDownloads(new Set());
    };

    return (
        <div className="glass rounded-xl flex-1 flex flex-col overflow-hidden">
            <div className="p-4 border-b border-white/5 flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <button onClick={selectAllDownloads} className="text-secondary hover:text-white">
                        {selectedDownloads.size === downloads.length && downloads.length > 0 ? <CheckSquare size={20} /> : <Square size={20} />}
                    </button>
                    <span className="text-sm text-secondary">{downloadTotal} total downloads</span>
                </div>
                <div className="flex gap-2">
                    {selectedDownloads.size > 0 && (
                        <>
                            <button
                                onClick={() => handleBulkDelete(false)}
                                className="px-3 py-1.5 bg-white/10 text-secondary rounded-lg text-sm font-medium hover:bg-white/20"
                            >
                                Clear Selected
                            </button>
                            <button
                                onClick={() => handleBulkDelete(true)}
                                className="px-3 py-1.5 bg-error/20 text-error rounded-lg text-sm font-medium hover:bg-error/30"
                            >
                                Delete Files ({selectedDownloads.size})
                            </button>
                        </>
                    )}
                    <button
                        onClick={handleClearAll}
                        disabled={downloadTotal === 0}
                        className={`px-3 py-1.5 rounded-lg text-sm font-medium ${downloadTotal > 0 ? 'bg-error/20 text-error hover:bg-error/30' : 'bg-white/5 text-secondary opacity-50 cursor-not-allowed'}`}
                    >
                        Clear All
                    </button>
                    <button onClick={fetchDownloadHistory} className="p-2 text-secondary hover:text-white">
                        <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
                    </button>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto">
                {downloads.length === 0 ? (
                    <div className="text-center py-12 text-muted">No download history</div>
                ) : (
                    <table className="w-full">
                        <thead className="bg-white/5 sticky top-0">
                            <tr className="text-xs uppercase text-secondary font-semibold">
                                <th className="px-4 py-3 text-left w-12"></th>
                                <th className="px-4 py-3 text-left">Filename</th>
                                <th className="px-4 py-3 text-left">Size</th>
                                <th className="px-4 py-3 text-left">Status</th>
                                <th className="px-4 py-3 text-left">Date</th>
                                <th className="px-4 py-3 text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-white/5">
                            {downloads.map(item => (
                                <tr key={item.id} className="hover:bg-white/5">
                                    <td className="px-4 py-3">
                                        <button onClick={() => toggleDownloadSelection(item.id)} className="text-secondary hover:text-white">
                                            {selectedDownloads.has(item.id) ? <CheckSquare size={18} /> : <Square size={18} />}
                                        </button>
                                    </td>
                                    <td className="px-4 py-3 text-sm font-medium truncate max-w-xs cursor-pointer hover:text-primary transition-colors flex items-center gap-2" title={item.file_name || `Pack #${item.slot} from ${item.bot}`} onClick={() => setSelectedDownloadLog(item)}>
                                        {item.file_name || (item.slot ? `Pack #${item.slot} from ${item.bot}` : 'Unknown')}
                                        <Terminal size={14} className="text-secondary opacity-50" />
                                    </td>
                                    <td className="px-4 py-3 text-secondary text-sm">
                                        {item.size ? formatBytes(item.size) : '-'}
                                    </td>
                                    <td className="px-4 py-3">
                                        <span className={`px-2 py-1 rounded text-xs font-medium ${item.status === 'completed' ? 'bg-success/20 text-success' : 'bg-error/20 text-error'
                                            }`}>
                                            {item.status}
                                        </span>
                                    </td>
                                    <td className="px-4 py-3 text-secondary text-sm">
                                        {new Date(item.completed_at).toLocaleString()}
                                    </td>
                                    <td className="px-4 py-3 text-right">
                                        <div className="flex justify-end gap-1">
                                            <button
                                                onClick={() => deleteDownload(item.id, false)}
                                                className="p-1.5 text-secondary hover:text-white rounded"
                                                title="Remove from history"
                                            >
                                                <X size={16} />
                                            </button>
                                            <button
                                                onClick={() => {
                                                    if (window.confirm(`Delete "${item.file_name}" from disk?`)) {
                                                        deleteDownload(item.id, true);
                                                    }
                                                }}
                                                className="p-1.5 text-error hover:bg-error hover:text-white rounded"
                                                title="Delete file"
                                            >
                                                <Trash2 size={16} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>

            {downloadTotalPages > 1 && (
                <div className="p-4 border-t border-white/5 flex items-center justify-center gap-4">
                    <button
                        onClick={() => setDownloadPage(p => Math.max(1, p - 1))}
                        disabled={downloadPage === 1}
                        className="p-2 text-secondary hover:text-white disabled:opacity-50"
                    >
                        <ChevronLeft size={20} />
                    </button>
                    <span className="text-sm text-secondary">
                        Page {downloadPage} of {downloadTotalPages}
                    </span>
                    <button
                        onClick={() => setDownloadPage(p => Math.min(downloadTotalPages, p + 1))}
                        disabled={downloadPage === downloadTotalPages}
                        className="p-2 text-secondary hover:text-white disabled:opacity-50"
                    >
                        <ChevronRight size={20} />
                    </button>
                </div>
            )}

            {selectedDownloadLog && (
                <TransferLogsModal
                    transferId={selectedDownloadLog.id}
                    fileName={selectedDownloadLog.file_name}
                    bot={selectedDownloadLog.bot}
                    onClose={() => setSelectedDownloadLog(null)}
                />
            )}
        </div>
    );
};
