import React, { useState, useEffect, useCallback } from 'react';
import { formatBytes } from '../utils/format';
import { useToast } from '../hooks/useToast';
import { Search, Download, Trash2, X, ChevronLeft, ChevronRight, CheckSquare, Square, RefreshCw } from 'lucide-react';

interface SearchHistoryItem {
    id: number;
    query: string;
    results_count: number;
    results_json?: string;
    searched_at: string;
}

interface DownloadHistoryItem {
    id: string;
    file_name?: string;
    size?: number;
    network: string;
    bot: string;
    status: string;
    completed_at: string;
}

interface PaginatedResponse<T> {
    items: T[];
    total: number;
    page: number;
    limit: number;
    total_pages: number;
}

// function interface moved


export const HistoryTab: React.FC = () => {
    const { showToast } = useToast();
    const [activeSection, setActiveSection] = useState<'downloads' | 'searches'>('downloads');

    // Search history state
    const [searches, setSearches] = useState<SearchHistoryItem[]>([]);
    const [searchPage, setSearchPage] = useState(1);
    const [searchTotalPages, setSearchTotalPages] = useState(1);
    const [searchTotal, setSearchTotal] = useState(0);
    const [selectedSearches, setSelectedSearches] = useState<Set<number>>(new Set());
    const [expandedSearch, setExpandedSearch] = useState<number | null>(null);

    // Download history will be loaded from the database API
    const [downloads, setDownloads] = useState<DownloadHistoryItem[]>([]);
    const [downloadPage, setDownloadPage] = useState(1);
    const [downloadTotalPages, setDownloadTotalPages] = useState(1);
    const [downloadTotal, setDownloadTotal] = useState(0);
    const [selectedDownloads, setSelectedDownloads] = useState<Set<string>>(new Set());

    const [loading, setLoading] = useState(false);

    const fetchSearchHistory = useCallback(async () => {
        setLoading(true);
        try {
            const res = await fetch(`/api/search-history?page=${searchPage}&limit=10`);
            const data: PaginatedResponse<SearchHistoryItem> = await res.json();
            setSearches(data.items);
            setSearchTotalPages(data.total_pages);
            setSearchTotal(data.total);
        } catch (e) {
            console.error('Failed to fetch search history:', e);
        }
        setLoading(false);
    }, [searchPage]);

    const fetchDownloadHistory = useCallback(async () => {
        setLoading(true);
        try {
            const res = await fetch(`/api/history?page=${downloadPage}&limit=10`);
            const data = await res.json();
            // The current API returns { history: [...], count: n }
            // We'll handle both formats
            if (data.items) {
                setDownloads(data.items);
                setDownloadTotalPages(data.total_pages || 1);
                setDownloadTotal(data.total || data.items.length);
            } else if (data.history) {
                setDownloads(data.history);
                setDownloadTotalPages(1);
                setDownloadTotal(data.count || data.history.length);
            }
        } catch (e) {
            console.error('Failed to fetch download history:', e);
        }
        setLoading(false);
    }, [downloadPage]);

    useEffect(() => {
        if (activeSection === 'searches') {
            fetchSearchHistory();
        } else {
            fetchDownloadHistory();
        }
    }, [activeSection, fetchSearchHistory, fetchDownloadHistory]);

    const handleDeleteSearch = async (id: number) => {
        try {
            await fetch(`/api/search-history/${id}`, { method: 'DELETE' });
            setSearches(prev => prev.filter(s => s.id !== id));
            setSearchTotal(prev => prev - 1);
        } catch (e) {
            console.error('Failed to delete search:', e);
        }
    };

    const handleBulkDeleteSearches = async () => {
        if (selectedSearches.size === 0) return;
        if (!window.confirm(`Delete ${selectedSearches.size} search history items?`)) return;

        try {
            await fetch('/api/search-history/bulk', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ids: Array.from(selectedSearches) }),
            });
            setSearches(prev => prev.filter(s => !selectedSearches.has(s.id)));
            setSearchTotal(prev => prev - selectedSearches.size);
            setSelectedSearches(new Set());
        } catch (e) {
            console.error('Failed to bulk delete searches:', e);
        }
    };

    const handleDownload = async (result: any) => {
        const url = `irc://${result.server}/${result.channel}/${result.bot}/${result.pack_number}`;
        try {
            await fetch('/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url })
            });
            showToast("Download started", "success");
        } catch (e) {
            console.error("Download start failed", e);
            showToast("Failed to start download", "error");
        }
    };

    const handleBulkDeleteDownloads = async (deleteFiles: boolean) => {
        if (selectedDownloads.size === 0) return;
        const msg = deleteFiles
            ? `Delete ${selectedDownloads.size} items AND their files from disk?`
            : `Remove ${selectedDownloads.size} items from history?`;
        if (!window.confirm(msg)) return;

        try {
            await fetch('/api/history/bulk', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ids: Array.from(selectedDownloads), delete_files: deleteFiles }),
            });
            setDownloads(prev => prev.filter(d => !selectedDownloads.has(d.id)));
            setDownloadTotal(prev => prev - selectedDownloads.size);
            setSelectedDownloads(new Set());
        } catch (e) {
            console.error('Failed to bulk delete downloads:', e);
        }
    };



    const handleDeleteDownload = async (id: string, deleteFile: boolean) => {
        try {
            await fetch(`/api/history/${id}?delete_file=${deleteFile}`, {
                method: 'DELETE',
            });
            setDownloads(prev => prev.filter(item => item.id !== id));
            setDownloadTotal(prev => prev - 1);
        } catch (e) {
            console.error('Failed to delete download:', e);
        }
    };

    const toggleSearchSelection = (id: number) => {
        setSelectedSearches(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    const toggleDownloadSelection = (id: string) => {
        setSelectedDownloads(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    const selectAllSearches = () => {
        if (selectedSearches.size === searches.length) {
            setSelectedSearches(new Set());
        } else {
            setSelectedSearches(new Set(searches.map(s => s.id)));
        }
    };

    const selectAllDownloads = () => {
        if (selectedDownloads.size === downloads.length) {
            setSelectedDownloads(new Set());
        } else {
            setSelectedDownloads(new Set(downloads.map(d => d.id)));
        }
    };

    return (
        <div className="flex flex-col h-full animate-fade-in">
            <div className="flex items-center justify-between mb-6">
                <h1 className="text-2xl font-bold">History</h1>
                <div className="flex gap-2">
                    <button
                        onClick={() => setActiveSection('downloads')}
                        className={`px-4 py-2 rounded-lg font-medium transition-colors flex items-center gap-2 ${activeSection === 'downloads'
                            ? 'bg-primary text-white'
                            : 'bg-white/5 text-secondary hover:text-white'
                            }`}
                    >
                        <Download size={18} />
                        Downloads
                    </button>
                    <button
                        onClick={() => setActiveSection('searches')}
                        className={`px-4 py-2 rounded-lg font-medium transition-colors flex items-center gap-2 ${activeSection === 'searches'
                            ? 'bg-primary text-white'
                            : 'bg-white/5 text-secondary hover:text-white'
                            }`}
                    >
                        <Search size={18} />
                        Searches
                    </button>
                </div>
            </div>

            {activeSection === 'searches' && (
                <div className="glass rounded-xl flex-1 flex flex-col overflow-hidden">
                    <div className="p-4 border-b border-white/5 flex items-center justify-between">
                        <div className="flex items-center gap-4">
                            <button onClick={selectAllSearches} className="text-secondary hover:text-white">
                                {selectedSearches.size === searches.length && searches.length > 0 ? <CheckSquare size={20} /> : <Square size={20} />}
                            </button>
                            <span className="text-sm text-secondary">{searchTotal} total searches</span>
                        </div>
                        <div className="flex gap-2">
                            {selectedSearches.size > 0 && (
                                <button
                                    onClick={handleBulkDeleteSearches}
                                    className="px-3 py-1.5 bg-error/20 text-error rounded-lg text-sm font-medium hover:bg-error/30"
                                >
                                    Delete Selected ({selectedSearches.size})
                                </button>
                            )}
                            <button onClick={fetchSearchHistory} className="p-2 text-secondary hover:text-white">
                                <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
                            </button>
                        </div>
                    </div>

                    <div className="flex-1 overflow-y-auto">
                        {searches.length === 0 ? (
                            <div className="text-center py-12 text-muted">No search history</div>
                        ) : (
                            <table className="w-full">
                                <thead className="bg-white/5 sticky top-0">
                                    <tr className="text-xs uppercase text-secondary font-semibold">
                                        <th className="px-4 py-3 text-left w-12"></th>
                                        <th className="px-4 py-3 text-left">Query</th>
                                        <th className="px-4 py-3 text-left">Results</th>
                                        <th className="px-4 py-3 text-left">Date</th>
                                        <th className="px-4 py-3 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-white/5">
                                    {searches.map(item => (
                                        <React.Fragment key={item.id}>
                                            <tr className="hover:bg-white/5">
                                                <td className="px-4 py-3">
                                                    <button onClick={() => toggleSearchSelection(item.id)} className="text-secondary hover:text-white">
                                                        {selectedSearches.has(item.id) ? <CheckSquare size={18} /> : <Square size={18} />}
                                                    </button>
                                                </td>
                                                <td className="px-4 py-3 font-mono text-sm">{item.query}</td>
                                                <td className="px-4 py-3">
                                                    <button
                                                        onClick={() => setExpandedSearch(expandedSearch === item.id ? null : item.id)}
                                                        className="text-primary hover:text-white underline"
                                                    >
                                                        {item.results_count} results
                                                    </button>
                                                </td>
                                                <td className="px-4 py-3 text-secondary text-sm">
                                                    {new Date(item.searched_at).toLocaleString()}
                                                </td>
                                                <td className="px-4 py-3 text-right">
                                                    <button
                                                        onClick={() => handleDeleteSearch(item.id)}
                                                        className="p-1.5 text-secondary hover:text-error rounded"
                                                        title="Delete"
                                                    >
                                                        <X size={16} />
                                                    </button>
                                                </td>
                                            </tr>
                                            {expandedSearch === item.id && item.results_json && (
                                                <tr className="bg-white/5">
                                                    <td colSpan={5} className="px-4 py-4">
                                                        <div className="max-h-64 overflow-y-auto">
                                                            <table className="w-full text-sm">
                                                                <thead>
                                                                    <tr className="text-xs uppercase text-secondary">
                                                                        <th className="text-left py-2">Filename</th>
                                                                        <th className="text-left py-2">Size</th>
                                                                        <th className="text-left py-2">Bot</th>
                                                                        <th className="text-left py-2">Network</th>
                                                                        <th className="text-right py-2">Action</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    {JSON.parse(item.results_json).map((r: any, i: number) => (
                                                                        <tr key={i} className="border-t border-white/5">
                                                                            <td className="py-2 truncate max-w-xs" title={r.file_name}>{r.file_name}</td>
                                                                            <td className="py-2 text-secondary">{r.size_str}</td>
                                                                            <td className="py-2 text-secondary">{r.bot}</td>
                                                                            <td className="py-2 text-secondary">{r.server}</td>
                                                                            <td className="py-2 text-right">
                                                                                <button
                                                                                    onClick={() => handleDownload(r)}
                                                                                    className="p-1.5 text-primary hover:bg-primary/20 rounded transition-colors"
                                                                                    title="Download"
                                                                                >
                                                                                    <Download size={16} />
                                                                                </button>
                                                                            </td>
                                                                        </tr>
                                                                    ))}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    </td>
                                                </tr>
                                            )}
                                        </React.Fragment>
                                    ))}
                                </tbody>
                            </table>
                        )}
                    </div>

                    {searchTotalPages > 1 && (
                        <div className="p-4 border-t border-white/5 flex items-center justify-center gap-4">
                            <button
                                onClick={() => setSearchPage(p => Math.max(1, p - 1))}
                                disabled={searchPage === 1}
                                className="p-2 text-secondary hover:text-white disabled:opacity-50"
                            >
                                <ChevronLeft size={20} />
                            </button>
                            <span className="text-sm text-secondary">
                                Page {searchPage} of {searchTotalPages}
                            </span>
                            <button
                                onClick={() => setSearchPage(p => Math.min(searchTotalPages, p + 1))}
                                disabled={searchPage === searchTotalPages}
                                className="p-2 text-secondary hover:text-white disabled:opacity-50"
                            >
                                <ChevronRight size={20} />
                            </button>
                        </div>
                    )}
                </div>
            )}

            {activeSection === 'downloads' && (
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
                                        onClick={() => handleBulkDeleteDownloads(false)}
                                        className="px-3 py-1.5 bg-white/10 text-secondary rounded-lg text-sm font-medium hover:bg-white/20"
                                    >
                                        Clear Selected
                                    </button>
                                    <button
                                        onClick={() => handleBulkDeleteDownloads(true)}
                                        className="px-3 py-1.5 bg-error/20 text-error rounded-lg text-sm font-medium hover:bg-error/30"
                                    >
                                        Delete Files ({selectedDownloads.size})
                                    </button>
                                </>
                            )}
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
                                            <td className="px-4 py-3 text-sm font-medium truncate max-w-xs" title={item.file_name}>
                                                {item.file_name || 'Unknown'}
                                            </td>
                                            <td className="px-4 py-3 text-secondary text-sm">
                                                {formatBytes(item.size || 0)}
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
                                                        onClick={() => handleDeleteDownload(item.id, false)}
                                                        className="p-1.5 text-secondary hover:text-white rounded"
                                                        title="Remove from history"
                                                    >
                                                        <X size={16} />
                                                    </button>
                                                    <button
                                                        onClick={() => {
                                                            if (window.confirm(`Delete "${item.file_name}" from disk?`)) {
                                                                handleDeleteDownload(item.id, true);
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
                </div>
            )}
        </div>
    );
};
