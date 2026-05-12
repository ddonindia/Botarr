import React, { useState } from 'react';
import { SearchHistoryItem } from '../../hooks/useHistory';
import { CheckSquare, Square, RefreshCw, X, Download, ChevronLeft, ChevronRight } from 'lucide-react';
import { useToast } from '../../hooks/useToast';

interface SearchHistoryProps {
    searches: SearchHistoryItem[];
    searchPage: number;
    setSearchPage: React.Dispatch<React.SetStateAction<number>>;
    searchTotalPages: number;
    searchTotal: number;
    fetchSearchHistory: () => Promise<void>;
    deleteSearch: (id: number) => Promise<void>;
    bulkDeleteSearches: (selectedIds: Set<number>) => Promise<void>;
    clearAllSearches: () => Promise<void>;
    loading: boolean;
}

export const SearchHistory: React.FC<SearchHistoryProps> = ({
    searches,
    searchPage,
    setSearchPage,
    searchTotalPages,
    searchTotal,
    fetchSearchHistory,
    deleteSearch,
    bulkDeleteSearches,
    clearAllSearches,
    loading
}) => {
    const { showToast } = useToast();
    const [selectedSearches, setSelectedSearches] = useState<Set<number>>(new Set());
    const [expandedSearch, setExpandedSearch] = useState<number | null>(null);

    const toggleSearchSelection = (id: number) => {
        setSelectedSearches(prev => {
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

    const handleBulkDelete = async () => {
        if (selectedSearches.size === 0) return;
        if (!window.confirm(`Delete ${selectedSearches.size} search history items?`)) return;
        await bulkDeleteSearches(selectedSearches);
        setSelectedSearches(new Set());
    };

    const handleClearAll = async () => {
        if (!window.confirm(`Are you sure you want to clear ALL search history?`)) return;
        await clearAllSearches();
        setSelectedSearches(new Set());
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

    return (
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
                            onClick={handleBulkDelete}
                            className="px-3 py-1.5 bg-error/20 text-error rounded-lg text-sm font-medium hover:bg-error/30"
                        >
                            Delete Selected ({selectedSearches.size})
                        </button>
                    )}
                    <button
                        onClick={handleClearAll}
                        disabled={searchTotal === 0}
                        className={`px-3 py-1.5 rounded-lg text-sm font-medium ${searchTotal > 0 ? 'bg-error/20 text-error hover:bg-error/30' : 'bg-white/5 text-secondary opacity-50 cursor-not-allowed'}`}
                    >
                        Clear All
                    </button>
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
                                                onClick={() => deleteSearch(item.id)}
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
    );
};
