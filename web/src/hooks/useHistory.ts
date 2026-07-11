import { useState, useCallback } from 'react';
import { useToast } from './useToast';

export interface SearchHistoryItem {
    id: number;
    query: string;
    results_count: number;
    results_json?: string;
    searched_at: string;
}

export interface DownloadHistoryItem {
    id: string;
    file_name?: string;
    size?: number;
    network: string;
    bot: string;
    channel: string;
    slot: number;
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

export const useHistory = () => {
    // Search history state
    const [searches, setSearches] = useState<SearchHistoryItem[]>([]);
    const [searchPage, setSearchPage] = useState(1);
    const [searchTotalPages, setSearchTotalPages] = useState(1);
    const [searchTotal, setSearchTotal] = useState(0);

    // Download history state
    const [downloads, setDownloads] = useState<DownloadHistoryItem[]>([]);
    const [downloadPage, setDownloadPage] = useState(1);
    const [downloadTotalPages, setDownloadTotalPages] = useState(1);
    const [downloadTotal, setDownloadTotal] = useState(0);

    const [loading, setLoading] = useState(false);
    const { showToast } = useToast();

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

    const deleteSearch = async (id: number) => {
        try {
            await fetch(`/api/search-history/${id}`, { method: 'DELETE' });
            setSearches(prev => prev.filter(s => s.id !== id));
            setSearchTotal(prev => prev - 1);
        } catch (e) {
            console.error('Failed to delete search:', e);
        }
    };

    const bulkDeleteSearches = async (selectedIds: Set<number>) => {
        if (selectedIds.size === 0) return;
        try {
            await fetch('/api/search-history/bulk', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ids: Array.from(selectedIds) }),
            });
            setSearches(prev => prev.filter(s => !selectedIds.has(s.id)));
            setSearchTotal(prev => prev - selectedIds.size);
        } catch (e) {
            console.error('Failed to bulk delete searches:', e);
        }
    };

    const clearAllSearches = async () => {
        try {
            await fetch('/api/search-history', { method: 'DELETE' });
            setSearches([]);
            setSearchTotal(0);
            setSearchPage(1);
            setSearchTotalPages(1);
        } catch (e) {
            console.error('Failed to clear all search history:', e);
        }
    };

    const deleteDownload = async (id: string, deleteFile: boolean) => {
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

    const retryDownload = async (item: DownloadHistoryItem) => {
        try {
            // Delete from history first to prevent duplicate errors
            const deleteRes = await fetch(`/api/history/${item.id}?delete_file=false`, {
                method: 'DELETE',
            });

            if (!deleteRes.ok) {
                showToast('Failed to clear old history entry', 'error');
                return;
            }

            // Re-submit as a new download
            const url = `irc://${item.network}/${item.channel}/${item.bot}/${item.slot}`;
            const res = await fetch('/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url, filename: item.file_name })
            });

            if (res.ok) {
                setDownloads(prev => prev.filter(d => d.id !== item.id));
                setDownloadTotal(prev => prev - 1);
                showToast('Retry started', 'success');
            } else {
                showToast('Failed to retry download', 'error');
            }
        } catch (e) {
            console.error('Failed to retry download:', e);
            showToast('Failed to retry download', 'error');
        }
    };

    const bulkDeleteDownloads = async (selectedIds: Set<string>, deleteFiles: boolean) => {
        if (selectedIds.size === 0) return;
        try {
            await fetch('/api/history/bulk', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ids: Array.from(selectedIds), delete_files: deleteFiles }),
            });
            setDownloads(prev => prev.filter(d => !selectedIds.has(d.id)));
            setDownloadTotal(prev => prev - selectedIds.size);
        } catch (e) {
            console.error('Failed to bulk delete downloads:', e);
        }
    };

    const clearAllDownloads = async () => {
        try {
            await fetch('/api/history', { method: 'DELETE' });
            setDownloads([]);
            setDownloadTotal(0);
            setDownloadPage(1);
            setDownloadTotalPages(1);
        } catch (e) {
            console.error('Failed to clear all download history:', e);
        }
    };

    return {
        searches,
        searchPage,
        setSearchPage,
        searchTotalPages,
        searchTotal,
        fetchSearchHistory,
        deleteSearch,
        bulkDeleteSearches,
        clearAllSearches,

        downloads,
        downloadPage,
        setDownloadPage,
        downloadTotalPages,
        downloadTotal,
        fetchDownloadHistory,
        deleteDownload,
        retryDownload,
        bulkDeleteDownloads,
        clearAllDownloads,

        loading
    };
};
