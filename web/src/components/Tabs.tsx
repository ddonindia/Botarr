import React from 'react';
import { Settings } from 'lucide-react';

export type TabType = 'search' | 'activities' | 'history' | 'plugins' | 'client' | 'settings';

interface TabsProps {
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

export const Tabs: React.FC<TabsProps> = ({ activeTab, onTabChange }) => {
    return (
        <div className="flex justify-center border-b border-white/5 mb-8">
            <button
                onClick={() => onTabChange('search')}
                className={`px-8 py-3 text-sm font-medium transition-all relative ${activeTab === 'search'
                    ? 'text-white'
                    : 'text-secondary hover:text-white'
                    }`}
            >
                Search
                {activeTab === 'search' && (
                    <div className="absolute bottom-0 left-0 w-full h-0.5 bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.5)]" />
                )}
            </button>
            <button
                onClick={() => onTabChange('activities')}
                className={`px-8 py-3 text-sm font-medium transition-all relative ${activeTab === 'activities'
                    ? 'text-white'
                    : 'text-secondary hover:text-white'
                    }`}
            >
                Activities
                {activeTab === 'activities' && (
                    <div className="absolute bottom-0 left-0 w-full h-0.5 bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.5)]" />
                )}
            </button>
            <button
                onClick={() => onTabChange('history')}
                className={`px-8 py-3 text-sm font-medium transition-all relative ${activeTab === 'history'
                    ? 'text-white'
                    : 'text-secondary hover:text-white'
                    }`}
            >
                History
                {activeTab === 'history' && (
                    <div className="absolute bottom-0 left-0 w-full h-0.5 bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.5)]" />
                )}
            </button>
            <button
                onClick={() => onTabChange('plugins')}
                className={`px-8 py-3 text-sm font-medium transition-all relative ${activeTab === 'plugins'
                    ? 'text-white'
                    : 'text-secondary hover:text-white'
                    }`}
            >
                Plugins
                {activeTab === 'plugins' && (
                    <div className="absolute bottom-0 left-0 w-full h-0.5 bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.5)]" />
                )}
            </button>
            <button
                onClick={() => onTabChange('client')}
                className={`flex-1 py-2 px-4 text-center border-b-2 font-medium text-sm transition-colors ${
                    activeTab === 'client'
                        ? 'border-blue-500 text-blue-500 bg-blue-500/10'
                        : 'border-transparent text-gray-400 hover:text-gray-300 hover:border-gray-600'
                }`}
            >
                IRC Client
            </button>
            <button
                onClick={() => onTabChange('settings')}
                className={`px-8 py-3 text-sm font-medium transition-all relative flex items-center gap-2 ${activeTab === 'settings'
                    ? 'text-white'
                    : 'text-secondary hover:text-white'
                    }`}
            >
                <Settings size={16} />
                Settings
                {activeTab === 'settings' && (
                    <div className="absolute bottom-0 left-0 w-full h-0.5 bg-primary shadow-[0_0_10px_rgba(var(--primary-rgb),0.5)]" />
                )}
            </button>
        </div>
    );
};
