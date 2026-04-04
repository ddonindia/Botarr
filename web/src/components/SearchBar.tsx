import React, { useState } from 'react';
import { Search, Loader2 } from 'lucide-react';

interface SearchBarProps {
    onSearch: (query: string, providers?: string[]) => void;
    isLoading: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({ onSearch, isLoading }) => {
    const [query, setQuery] = useState('');
    const [selectedProviders, setSelectedProviders] = useState<string[]>([]);
    const [showFilters, setShowFilters] = useState(false);

    const providers = [
        { id: 'SkullXDCC', name: 'SkullXDCC' },
        { id: 'XDCC.rocks', name: 'XDCC.rocks' },
        { id: 'XDCC.eu', name: 'XDCC.eu' },
    ];

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (query.trim()) {
            onSearch(query, selectedProviders);
        }
    };

    const toggleProvider = (id: string) => {
        setSelectedProviders(prev => {
            if (prev.includes(id)) {
                return prev.filter(p => p !== id);
            } else {
                return [...prev, id];
            }
        });
    };

    return (
        <div className="w-full max-w-4xl mx-auto mb-10">
            <form onSubmit={handleSubmit} className="relative group z-20">
                <div className="absolute -inset-1 bg-gradient-to-r from-primary to-purple-600 rounded-lg blur opacity-25 group-hover:opacity-75 transition duration-1000 group-hover:duration-200"></div>
                <div className="relative flex items-center">
                    <Search className="absolute left-4 text-muted w-5 h-5" />
                    <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search for packs (e.g., '1080p linux iso')..."
                        className="w-full bg-surface border border-white/10 rounded-lg py-4 pl-12 pr-32 text-white placeholder-muted focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all font-medium"
                    />

                    <div className="absolute right-2 flex items-center gap-2">
                        <div className="relative">
                            <button
                                type="button"
                                onClick={() => setShowFilters(!showFilters)}
                                className={`px-3 py-2 rounded-md text-sm font-medium transition-colors border border-white/10 ${showFilters || selectedProviders.length > 0 ? 'bg-primary/20 text-primary border-primary/30' : 'bg-surface hover:bg-white/5 text-muted'}`}
                            >
                                {selectedProviders.length === 0 ? 'All Providers' : `${selectedProviders.length} Selected`}
                            </button>

                            {showFilters && (
                                <div className="absolute top-12 right-0 w-48 bg-surface border border-white/10 rounded-lg shadow-xl p-2 flex flex-col gap-1">
                                    <div className="text-xs font-semibold text-muted px-2 py-1 uppercase tracking-wider">Search Providers</div>
                                    {providers.map(p => (
                                        <label key={p.id} className="flex items-center gap-2 px-2 py-1.5 hover:bg-white/5 rounded cursor-pointer">
                                            <input
                                                type="checkbox"
                                                checked={selectedProviders.includes(p.id)}
                                                onChange={() => toggleProvider(p.id)}
                                                className="rounded border-white/20 bg-black/20 text-primary focus:ring-primary/50"
                                            />
                                            <span className="text-sm text-gray-300">{p.name}</span>
                                        </label>
                                    ))}
                                    <div className="border-t border-white/10 my-1"></div>
                                    <button
                                        type="button"
                                        onClick={() => setSelectedProviders([])}
                                        className="text-xs text-center py-1 text-secondary hover:text-white"
                                    >
                                        Reset to All
                                    </button>
                                </div>
                            )}
                        </div>

                        <button
                            type="submit"
                            disabled={isLoading}
                            className="bg-primary hover:bg-primaryHover text-white px-4 py-2 rounded-md font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                        >
                            {isLoading ? <Loader2 className="animate-spin w-4 h-4" /> : 'Search'}
                        </button>
                    </div>
                </div>
            </form>

            {selectedProviders.length > 0 && (
                <div className="mt-2 flex gap-2 justify-center">
                    <span className="text-xs text-muted">Searching in:</span>
                    {selectedProviders.map(p => (
                        <span key={p} className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded-full border border-primary/20">
                            {p}
                        </span>
                    ))}
                </div>
            )}
        </div>
    );
};
