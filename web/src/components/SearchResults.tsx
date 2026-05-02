import React, { useState, useMemo } from 'react';
import { XdccSearchResult } from '../types';
import { formatBytes } from '../utils/format';
import { Filter, Grid, List } from 'lucide-react';

interface SearchResultsProps {
    results: XdccSearchResult[];
    onDownload: (result: XdccSearchResult) => void;
    onQueueDownload?: (result: XdccSearchResult) => void;
}

export const SearchResults: React.FC<SearchResultsProps> = ({ results, onDownload, onQueueDownload }) => {
    const [filterQuery, setFilterQuery] = useState('');
    const [minSize, setMinSize] = useState<number | ''>('');
    const [maxSize, setMaxSize] = useState<number | ''>('');
    const [selectedServer, setSelectedServer] = useState<string>('');
    const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

    // Normalize server/network names for deduplication
    const normalizeServer = (s: string) => {
        let norm = s.toLowerCase();
        // Strip common prefixes
        norm = norm.replace(/^irc\./, '');
        // Keep full name for display but dedupe on normalized form
        return norm;
    };

    const servers = useMemo(() => {
        // Create map of normalized -> original (keeping first occurrence)
        const seen = new Map<string, string>();
        results.forEach(r => {
            const norm = normalizeServer(r.server);
            if (!seen.has(norm)) {
                seen.set(norm, r.server);
            }
        });
        return Array.from(seen.values()).sort();
    }, [results]);

    const filteredResults = useMemo(() => {
        return results.filter(res => {
            // Text filter
            if (filterQuery && !res.file_name.toLowerCase().includes(filterQuery.toLowerCase())) {
                return false;
            }

            // Server filter (use normalized comparison)
            if (selectedServer && normalizeServer(res.server) !== normalizeServer(selectedServer)) {
                return false;
            }

            // Size filter (in bytes)
            // Assuming simplified inputs for now (MB)
            if (minSize !== '' && res.file_size < minSize * 1024 * 1024) return false;
            if (maxSize !== '' && res.file_size > maxSize * 1024 * 1024) return false;

            return true;
        });
    }, [results, filterQuery, selectedServer, minSize, maxSize]);

    return (
        <div className="animate-fade-in">
            {/* Filter Bar */}
            <div className="glass p-4 rounded-xl mb-6 flex flex-wrap gap-4 items-center">
                <div className="flex items-center gap-2 text-secondary">
                    <Filter size={18} />
                    <span className="text-sm font-medium">Filters</span>
                </div>

                <input
                    type="text"
                    placeholder="Filter results..."
                    value={filterQuery}
                    onChange={(e) => setFilterQuery(e.target.value)}
                    className="bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white placeholder-muted focus:outline-none focus:border-primary/50 transition-colors"
                />

                <select
                    value={selectedServer}
                    onChange={(e) => setSelectedServer(e.target.value)}
                    className="bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50 transition-colors appearance-none cursor-pointer"
                >
                    <option value="">All Servers</option>
                    {servers.map(s => (
                        <option key={s} value={s}>{s}</option>
                    ))}
                </select>

                <div className="flex items-center gap-2">
                    <input
                        type="number"
                        placeholder="Min MB"
                        value={minSize}
                        onChange={(e) => setMinSize(e.target.value ? Number(e.target.value) : '')}
                        className="bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white placeholder-muted w-24 focus:outline-none focus:border-primary/50 transition-colors"
                    />
                    <span className="text-muted text-xs">-</span>
                    <input
                        type="number"
                        placeholder="Max MB"
                        value={maxSize}
                        onChange={(e) => setMaxSize(e.target.value ? Number(e.target.value) : '')}
                        className="bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white placeholder-muted w-24 focus:outline-none focus:border-primary/50 transition-colors"
                    />
                </div>

                <div className="flex items-center gap-2 bg-surface border border-white/10 rounded p-1 ml-auto">
                    <button 
                        onClick={() => setViewMode('grid')}
                        className={`p-1 rounded ${viewMode === 'grid' ? 'bg-primary/20 text-primary' : 'text-muted hover:text-white'}`}
                        title="Grid View"
                    >
                        <Grid size={16} />
                    </button>
                    <button 
                        onClick={() => setViewMode('list')}
                        className={`p-1 rounded ${viewMode === 'list' ? 'bg-primary/20 text-primary' : 'text-muted hover:text-white'}`}
                        title="List View"
                    >
                        <List size={16} />
                    </button>
                </div>
                
                <div className="text-xs text-muted">
                    Showing {filteredResults.length} / {results.length} results
                </div>
            </div>

            {/* Results */}
            <div className="overflow-y-auto max-h-[70vh] pr-2 custom-scrollbar">
                {viewMode === 'list' ? (
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-sm text-white/80">
                            <thead className="text-xs text-secondary border-b border-white/10">
                                <tr>
                                    <th className="py-2 px-4 font-medium">Filename</th>
                                    <th className="py-2 px-4 font-medium">Size</th>
                                    <th className="py-2 px-4 font-medium">Bot</th>
                                    <th className="py-2 px-4 font-medium">Server</th>
                                    <th className="py-2 px-4 font-medium text-right">Downloads</th>
                                    <th className="py-2 px-4 font-medium text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filteredResults.map((res, i) => (
                                    <tr key={i} className="border-b border-white/5 hover:bg-white/5 transition-colors group cursor-pointer" onClick={() => onDownload(res)}>
                                        <td className="py-3 px-4 max-w-[300px] truncate" title={res.file_name}>
                                            <span className="font-medium text-primary group-hover:underline">{res.file_name}</span>
                                        </td>
                                        <td className="py-3 px-4 whitespace-nowrap">{formatBytes(res.file_size)}</td>
                                        <td className="py-3 px-4 whitespace-nowrap"><span className="text-xs bg-white/5 px-2 py-1 rounded">{res.bot} #{res.pack_number}</span></td>
                                        <td className="py-3 px-4 whitespace-nowrap text-muted text-xs">{res.server}</td>
                                        <td className="py-3 px-4 text-right">{res.downloads}</td>
                                        <td className="py-3 px-4 text-right whitespace-nowrap">
                                            {onQueueDownload && (
                                                <button 
                                                    onClick={(e) => { e.stopPropagation(); onQueueDownload(res); }} 
                                                    className="text-primary hover:text-primary-light mr-3 text-xs"
                                                    title="Add to queue"
                                                >
                                                    Queue
                                                </button>
                                            )}
                                            <button 
                                                onClick={(e) => { e.stopPropagation(); onDownload(res); }} 
                                                className="text-primary hover:text-primary-light text-xs font-medium bg-primary/20 px-2 py-1 rounded"
                                            >
                                                Download
                                            </button>
                                        </td>
                                    </tr>
                                ))}
                                {filteredResults.length === 0 && (
                                    <tr>
                                        <td colSpan={6} className="py-12 text-center text-muted">
                                            No matching results found.
                                        </td>
                                    </tr>
                                )}
                            </tbody>
                        </table>
                    </div>
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {filteredResults.map((res, i) => (
                            <div key={i} className="glass-card p-4 group cursor-pointer hover:bg-white/5 transition-all" onClick={() => onDownload(res)}>
                                <div className="flex justify-between items-start mb-2">
                                    <h4 className="font-medium text-primary group-hover:underline truncate w-full" title={res.file_name}>
                                        {res.file_name}
                                    </h4>
                                </div>
                                <div className="flex justify-between text-xs text-secondary mt-2">
                                    <span>{formatBytes(res.file_size)}</span>
                                    <span>{res.downloads} dl</span>
                                </div>
                                <div className="mt-3 pt-3 border-t border-white/5 flex justify-between items-center">
                                    <span className="text-xs bg-white/5 px-2 py-1 rounded">{res.bot} #{res.pack_number}</span>
                                    <span className="text-xs text-muted">{res.server}</span>
                                </div>
                                {onQueueDownload && (
                                    <div className="mt-3 pt-3 border-t border-white/5 flex justify-end">
                                        <button 
                                            onClick={(e) => { e.stopPropagation(); onQueueDownload(res); }} 
                                            className="text-primary hover:text-primary-light text-xs mr-3"
                                        >
                                            Queue
                                        </button>
                                        <button 
                                            onClick={(e) => { e.stopPropagation(); onDownload(res); }} 
                                            className="text-primary hover:text-primary-light text-xs font-medium bg-primary/20 px-2 py-1 rounded"
                                        >
                                            Download
                                        </button>
                                    </div>
                                )}
                            </div>
                        ))}
                        {filteredResults.length === 0 && (
                            <div className="col-span-full py-12 text-center text-muted">
                                No matching results found.
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
};
