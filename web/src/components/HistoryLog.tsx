import React from 'react';
import { HistoryItem } from '../types';
import { formatBytes } from '../utils/format';
import { FileText, Clock } from 'lucide-react';

interface HistoryLogProps {
    history: HistoryItem[];
    onDelete: (id: string, deleteFile: boolean) => void;
}

export const HistoryLog: React.FC<HistoryLogProps> = ({ history, onDelete }) => {
    return (
        <div className="mt-8">
            <h2 className="text-xl font-bold text-white mb-4">Recent History</h2>
            <div className="glass rounded-xl overflow-hidden">
                <table className="w-full text-left">
                    <thead className="bg-white/5 text-xs uppercase text-secondary font-semibold">
                        <tr>
                            <th className="px-6 py-3">Filename</th>
                            <th className="px-6 py-3">Size</th>
                            <th className="px-6 py-3">Date</th>
                            <th className="px-6 py-3">Status</th>
                            <th className="px-6 py-3 text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-white/5">
                        {history.map((item) => (
                            <tr key={item.id} className="hover:bg-white/5 transition-colors">
                                <td className="px-6 py-4">
                                    <div className="flex items-center gap-3">
                                        <FileText className="text-muted w-4 h-4" />
                                        <span className="text-sm text-text font-medium truncate max-w-xs" title={item.file_name || 'Unknown'}>
                                            {item.file_name || 'Unknown'}
                                        </span>
                                    </div>
                                </td>
                                <td className="px-6 py-4 text-sm text-secondary">
                                    {formatBytes(item.size || 0)}
                                </td>
                                <td className="px-6 py-4 text-sm text-secondary flex items-center gap-2">
                                    <Clock className="w-3 h-3" />
                                    {/* Backend sends ISO string for updated_at */}
                                    {new Date(item.updated_at).toLocaleDateString()}
                                </td>
                                <td className="px-6 py-4">
                                    <span className="px-2 py-1 rounded text-xs font-medium bg-success/20 text-success border border-success/20">
                                        {item.status || 'Completed'}
                                    </span>
                                </td>
                                <td className="px-6 py-4 text-right">
                                    <div className="flex justify-end gap-2">
                                        <button
                                            onClick={() => onDelete(item.id, false)}
                                            className="p-1.5 text-secondary hover:text-white hover:bg-white/10 rounded transition-colors"
                                            title="Remove from history"
                                        >
                                            {/* X icon - just removes from list */}
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
                                        </button>
                                        <button
                                            onClick={() => {
                                                if (window.confirm(`Delete "${item.file_name || 'this file'}" permanently from disk? This cannot be undone.`)) {
                                                    onDelete(item.id, true);
                                                }
                                            }}
                                            className="p-1.5 text-error hover:text-white hover:bg-error rounded transition-colors"
                                            title="Delete file permanently"
                                        >
                                            {/* Trash icon - deletes file */}
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /><line x1="10" y1="11" x2="10" y2="17" /><line x1="14" y1="11" x2="14" y2="17" /></svg>
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                {history.length === 0 && (
                    <div className="text-center py-8 text-muted text-sm">No history available</div>
                )}
            </div>
        </div>
    );
};
