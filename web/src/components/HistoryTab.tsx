import React, { useState, useEffect } from 'react';
import { Search, Download } from 'lucide-react';
import { useHistory } from '../hooks/useHistory';
import { SearchHistory } from './history/SearchHistory';
import { DownloadHistory } from './history/DownloadHistory';

export const HistoryTab: React.FC = () => {
    const [activeSection, setActiveSection] = useState<'downloads' | 'searches'>('downloads');
    const historyState = useHistory();

    useEffect(() => {
        if (activeSection === 'searches') {
            historyState.fetchSearchHistory();
        } else {
            historyState.fetchDownloadHistory();
        }
    }, [activeSection, historyState.fetchSearchHistory, historyState.fetchDownloadHistory]);

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
                <SearchHistory
                    searches={historyState.searches}
                    searchPage={historyState.searchPage}
                    setSearchPage={historyState.setSearchPage}
                    searchTotalPages={historyState.searchTotalPages}
                    searchTotal={historyState.searchTotal}
                    fetchSearchHistory={historyState.fetchSearchHistory}
                    deleteSearch={historyState.deleteSearch}
                    bulkDeleteSearches={historyState.bulkDeleteSearches}
                    clearAllSearches={historyState.clearAllSearches}
                    loading={historyState.loading}
                />
            )}

            {activeSection === 'downloads' && (
                <DownloadHistory
                    downloads={historyState.downloads}
                    downloadPage={historyState.downloadPage}
                    setDownloadPage={historyState.setDownloadPage}
                    downloadTotalPages={historyState.downloadTotalPages}
                    downloadTotal={historyState.downloadTotal}
                    fetchDownloadHistory={historyState.fetchDownloadHistory}
                    deleteDownload={historyState.deleteDownload}
                    bulkDeleteDownloads={historyState.bulkDeleteDownloads}
                    clearAllDownloads={historyState.clearAllDownloads}
                    loading={historyState.loading}
                />
            )}
        </div>
    );
};
