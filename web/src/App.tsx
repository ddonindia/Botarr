import { useEffect, useState } from 'react';
import { SearchBar } from './components/SearchBar';
import { StatsBar } from './components/StatsBar';
import { TransferList } from './components/TransferList';
import { HistoryTab } from './components/HistoryTab';
import { SettingsTab } from './components/SettingsTab';
import { Toast } from './components/Toast';
import { Tabs } from './components/Tabs';
import { SearchResults } from './components/SearchResults';
import { useToast } from './hooks/useToast';
import { XdccSearchResult, XdccTransfer, BotStats } from './types';

function App() {
    const [activeTab, setActiveTab] = useState<'search' | 'activities' | 'history' | 'settings'>('search');
    const [searchResults, setSearchResults] = useState<XdccSearchResult[]>([]);
    const [transfers, setTransfers] = useState<XdccTransfer[]>([]);
    const [stats, setStats] = useState<BotStats[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [queueSize, setQueueSize] = useState(0);

    const { toast, showToast, hideToast } = useToast();

    useEffect(() => {
        const interval = setInterval(fetchUpdates, 1000);
        return () => clearInterval(interval);
    }, []);

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
            // Optional: Switch to activities tab on download start
            // setActiveTab('activities'); 
        } catch (e) {
            console.error("Download start failed", e);
            showToast("Failed to start download", "error");
        }
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

    // deleted handleDeleteHistory

    return (
        <div className="min-h-screen pb-12 flex flex-col">
            <header className="border-b border-white/5 glass sticky top-0 z-50">
                <div className="container mx-auto px-4 h-16 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <img src="/botarr.png" alt="Botarr" className="w-10 h-10 rounded-lg shadow-lg shadow-primary/20" />
                        <span className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">Botarr</span>
                    </div>

                </div>
            </header>

            <main className="container mx-auto px-4 pt-4 flex-1 flex flex-col">
                <Tabs activeTab={activeTab} onTabChange={setActiveTab} />

                {activeTab === 'search' && (
                    <div className="flex flex-col h-full animate-fade-in">
                        <div className="text-center mb-8">
                            <h1 className="text-3xl font-bold mb-2">Find & Download</h1>
                            <p className="text-secondary">Search across multiple XDCC bots and servers</p>
                        </div>

                        <SearchBar onSearch={handleSearch} isLoading={isLoading} />

                        {searchResults.length > 0 && (
                            <SearchResults results={searchResults} onDownload={handleDownload} />
                        )}
                    </div>
                )}

                {activeTab === 'activities' && (
                    <div className="flex flex-col gap-6 h-full overflow-hidden animate-fade-in">
                        <StatsBar
                            stats={stats}
                            queueSize={queueSize}
                            activeDownloads={transfers.filter(t =>
                                ['downloading', 'connecting', 'joining', 'requesting'].includes(t.status)
                            ).length}
                        />
                        <div className="flex-1 min-h-0">
                            <TransferList transfers={transfers} onCancel={handleCancel} onRetry={handleRetry} />
                        </div>
                    </div>
                )}

                {activeTab === 'history' && (
                    <HistoryTab />
                )}

                {activeTab === 'settings' && (
                    <SettingsTab />
                )}
            </main>

            {toast.visible && (
                <Toast message={toast.message} type={toast.type} onClose={hideToast} />
            )}
        </div>
    );
}

export default App;
