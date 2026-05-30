import { useEffect, useState, useRef } from 'react';
import { SearchBar } from './components/SearchBar';
import { StatsBar } from './components/StatsBar';
import { TransferList } from './components/TransferList';
import { HistoryTab } from './components/HistoryTab';
import { PluginsTab } from './components/PluginsTab';
import { IrcClientTab } from './components/IrcClientTab';
import { SettingsTab } from './components/SettingsTab';
import { Toast } from './components/Toast';
import { Tabs } from './components/Tabs';
import { SearchResults } from './components/SearchResults';
import { AutodlTab } from './components/AutodlTab';
import { ManualDownloadModal } from './components/ManualDownloadModal';
import { useToast } from './hooks/useToast';
import { XdccSearchResult, XdccTransfer, BotStats } from './types';

type TabType = 'search' | 'activities' | 'history' | 'plugins' | 'autodl' | 'client' | 'settings';

function App() {
    const [activeTab, setActiveTab] = useState<TabType>('search');
    const [searchResults, setSearchResults] = useState<XdccSearchResult[]>([]);
    const [transfers, setTransfers] = useState<XdccTransfer[]>([]);
    const [stats, setStats] = useState<BotStats[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [queueSize, setQueueSize] = useState(0);
    const [showManualDownload, setShowManualDownload] = useState(false);
    const [downloadQueue, setDownloadQueue] = useState<XdccSearchResult[]>([]);
    const isProcessingQueueRef = useRef(false);

    const { toast, showToast, hideToast } = useToast();

    useEffect(() => {
        const interval = setInterval(fetchUpdates, 1000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        const activeDownloads = transfers.filter(t =>
            ['downloading', 'connecting', 'joining', 'requesting', 'pending'].includes(t.status)
        ).length;

        if (activeDownloads === 0 && downloadQueue.length > 0 && !isProcessingQueueRef.current) {
            isProcessingQueueRef.current = true;
            const nextItem = downloadQueue[0];
            
            const url = `irc://${nextItem.server}/${nextItem.channel}/${nextItem.bot}/${nextItem.pack_number}`;
            fetch('/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url })
            }).then(() => {
                setDownloadQueue(prev => prev.slice(1));
                showToast("Started queued download", "success");
                fetchUpdates();
                setTimeout(() => {
                    isProcessingQueueRef.current = false;
                }, 1500);
            }).catch(e => {
                console.error("Download start failed", e);
                showToast("Failed to start queued download", "error");
                setDownloadQueue(prev => prev.slice(1));
                isProcessingQueueRef.current = false;
            });
        }
    }, [transfers, downloadQueue]);

    const fetchUpdates = async () => {
        try {
            // Fetch stats independently to fail gracefully
            try {
                const sRes = await fetch('/api/bots/stats').then(r => r.json());
                setStats(sRes.bots || []);
            } catch (e) {
                console.error("Stats fetch failed", e);
            }

            try {
                const qRes = await fetch('/api/queue').then(r => r.json());
                setQueueSize(qRes.queue_size || 0);
            } catch (e) {
                console.error("Queue fetch failed", e);
            }

            // Fetch transfers
            try {
                const tRes = await fetch('/api/transfers').then(r => r.json());
                setTransfers(tRes.transfers || []);
            } catch (e) {
                console.error("Transfers fetch failed", e);
            }
        } catch (e) {
            console.error("Unexpected error in fetchUpdates", e);
        }
    };

    const handleSearch = async (query: string, providers: string[] = []) => {
        setIsLoading(true);
        try {
            let url = `/api/search?query=${encodeURIComponent(query)}`;
            if (providers.length > 0) {
                url += `&providers=${encodeURIComponent(providers.join(','))}`;
            }
            const res = await fetch(url);
            const data = await res.json();
            setSearchResults(data.results);
        } catch (e) {
            console.error("Search failed", e);
            showToast("Search failed", "error");
        } finally {
            setIsLoading(false);
        }
    };

    const handleDownload = async (result: XdccSearchResult) => {
        const url = `irc://${result.server}/${result.channel}/${result.bot}/${result.pack_number}`;
        try {
            await fetch('/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url })
            });
            showToast("Download started", "success");
            fetchUpdates();
        } catch (e) {
            console.error("Download start failed", e);
            showToast("Failed to start download", "error");
        }
    };

    const handleManualDownload = async (url: string) => {
        try {
            const res = await fetch('/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url })
            });
            if (!res.ok) {
                const data = await res.json();
                throw new Error(data.error || "Failed to start download");
            }
            showToast("Download started", "success");
            fetchUpdates();
        } catch (e: any) {
            showToast(e.message || "Failed to start download", "error");
            throw e;
        }
    };

    const handleQueueDownload = (result: XdccSearchResult) => {
        setDownloadQueue(prev => [...prev, result]);
        showToast("Added to download queue", "success");
    };

    const handleCancel = async (id: string) => {
        await fetch(`/api/transfers/${id}`, { method: 'DELETE' });
        fetchUpdates();
        showToast("Transfer cancelled", "success");
    };

    const handleRetry = async (id: string) => {
        await fetch(`/api/transfers/${id}/retry`, { method: 'POST' });
        fetchUpdates();
        showToast("Retrying transfer...", "success");
    };

    const handleResume = async (id: string) => {
        await fetch(`/api/transfers/${id}/resume`, { method: 'POST' });
        fetchUpdates();
        showToast("Starting transfer...", "success");
    };

    const handleDelete = async (id: string) => {
        await fetch(`/api/transfers/${id}`, { method: 'DELETE' });
        fetchUpdates();
    };

    const handleClearFinished = async () => {
        const finished = transfers.filter(t =>
            ['completed', 'failed', 'cancelled'].includes(t.status)
        );
        await Promise.all(
            finished.map(t => fetch(`/api/transfers/${t.id}`, { method: 'DELETE' }))
        );
        fetchUpdates();
        showToast(`Cleared ${finished.length} finished transfer${finished.length !== 1 ? 's' : ''}`, "success");
    };

    return (
        <div className="min-h-screen pb-12 flex flex-col">
            <header className="border-b border-white/5 glass sticky top-0 z-50">
                <div className="container mx-auto px-4 h-16 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <img src="/botarr.png" alt="Botarr" className="w-10 h-10 rounded-lg shadow-lg shadow-primary/20" />
                        <span className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">Botarr</span>
                    </div>
                    <a
                        href="https://github.com/ddonindia/Botarr"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-white/60 hover:text-white transition-colors p-2 rounded-lg hover:bg-white/5"
                        aria-label="GitHub"
                    >
                        <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
                        </svg>
                    </a>
                </div>
            </header>

            <main className="container mx-auto px-4 pt-4 flex-1 flex flex-col">
                <Tabs activeTab={activeTab} onTabChange={setActiveTab} />

                {activeTab === 'search' && (
                    <div className="flex flex-col h-full animate-fade-in">
                        <div className="text-center mb-8 relative">
                            <h1 className="text-3xl font-bold mb-2">Find & Download</h1>
                            <p className="text-secondary">Search across multiple XDCC bots and servers</p>
                            <div className="absolute right-0 top-0">
                                <button 
                                    onClick={() => setShowManualDownload(true)} 
                                    className="btn btn-secondary flex items-center gap-2"
                                >
                                    <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
                                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                                        <polyline points="7 10 12 15 17 10"></polyline>
                                        <line x1="12" y1="15" x2="12" y2="3"></line>
                                    </svg>
                                    Manual Download
                                </button>
                            </div>
                        </div>

                        <SearchBar onSearch={handleSearch} isLoading={isLoading} />

                        {searchResults.length > 0 && (
                            <SearchResults 
                                results={searchResults} 
                                onDownload={handleDownload} 
                                onQueueDownload={handleQueueDownload}
                            />
                        )}
                        {downloadQueue.length > 0 && (
                            <div className="mt-4 text-center text-sm text-secondary">
                                {downloadQueue.length} item(s) in local download queue.
                            </div>
                        )}
                    </div>
                )}

                {activeTab === 'activities' && (
                    <div className="flex flex-col gap-6 h-full overflow-hidden animate-fade-in">
                        <StatsBar
                            stats={stats}
                            queueSize={queueSize + downloadQueue.length}
                            activeDownloads={transfers.filter(t =>
                                ['downloading', 'connecting', 'joining', 'requesting'].includes(t.status)
                            ).length}
                        />
                        <div className="flex-1 min-h-0">
                            <TransferList transfers={transfers} onCancel={handleCancel} onRetry={handleRetry} onResume={handleResume} onDelete={handleDelete} onClearFinished={handleClearFinished} />
                        </div>
                    </div>
                )}

                {activeTab === 'history' && (
                    <HistoryTab />
                )}

                {activeTab === 'plugins' && <PluginsTab />}
                {activeTab === 'autodl' && <AutodlTab />}
                {activeTab === 'client' && <IrcClientTab />}
                {activeTab === 'settings' && <SettingsTab />}
            </main>

            {toast.visible && (
                <Toast 
                    message={toast.message}
                    type={toast.type}
                    onClose={hideToast}
                />
            )}

            {showManualDownload && (
                <ManualDownloadModal 
                    onClose={() => setShowManualDownload(false)}
                    onSubmit={handleManualDownload}
                />
            )}
        </div>
    );
}

export default App;
