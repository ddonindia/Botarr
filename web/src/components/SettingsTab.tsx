import React, { useState } from 'react';
import { Settings, Wifi, User, Zap, Download, Search, Globe, Save, RefreshCw } from 'lucide-react';
import { useSettings } from '../hooks/useSettings';
import { GeneralSettings } from './settings/GeneralSettings';
import { NetworkSettings } from './settings/NetworkSettings';

type SettingsSection = 'connection' | 'identity' | 'behavior' | 'dcc' | 'search' | 'networks';

export const SettingsTab: React.FC = () => {
    const {
        settings,
        loading,
        saving,
        saveSettings,
        updateSetting,
        addNetwork,
        deleteNetwork,
        updateNetwork
    } = useSettings();

    const [activeSection, setActiveSection] = useState<SettingsSection>('connection');

    const sections: { id: SettingsSection; label: string; icon: React.ReactNode }[] = [
        { id: 'connection', label: 'Connection', icon: <Wifi size={18} /> },
        { id: 'identity', label: 'Identity', icon: <User size={18} /> },
        { id: 'behavior', label: 'Behavior', icon: <Zap size={18} /> },
        { id: 'dcc', label: 'DCC', icon: <Download size={18} /> },
        { id: 'search', label: 'Search', icon: <Search size={18} /> },
        { id: 'networks', label: 'Networks', icon: <Globe size={18} /> },
    ];

    if (loading) {
        return (
            <div className="flex items-center justify-center py-20">
                <RefreshCw className="animate-spin text-primary" size={32} />
            </div>
        );
    }

    if (!settings) {
        return (
            <div className="text-center py-20 text-muted">
                Failed to load settings
            </div>
        );
    }

    return (
        <div className="animate-fade-in">
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold flex items-center gap-2">
                    <Settings size={24} className="text-primary" />
                    Settings
                </h2>
                <button
                    onClick={saveSettings}
                    disabled={saving}
                    className="flex items-center gap-2 bg-primary hover:bg-primary/80 text-white px-4 py-2 rounded-lg transition-colors disabled:opacity-50"
                >
                    {saving ? <RefreshCw size={18} className="animate-spin" /> : <Save size={18} />}
                    Save Changes
                </button>
            </div>

            <div className="flex gap-6">
                {/* Sidebar */}
                <div className="w-48 shrink-0">
                    <div className="glass rounded-xl p-2 space-y-1">
                        {sections.map(section => (
                            <button
                                key={section.id}
                                onClick={() => setActiveSection(section.id)}
                                className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-left ${activeSection === section.id
                                    ? 'bg-primary/20 text-primary'
                                    : 'hover:bg-white/5 text-secondary'
                                    }`}
                            >
                                {section.icon}
                                <span className="text-sm font-medium">{section.label}</span>
                            </button>
                        ))}
                    </div>
                </div>

                {/* Content */}
                <div className="flex-1 glass rounded-xl p-6">
                    {activeSection !== 'networks' ? (
                        <GeneralSettings
                            settings={settings}
                            updateSetting={updateSetting}
                            activeSection={activeSection}
                        />
                    ) : (
                        <NetworkSettings
                            settings={settings}
                            updateNetwork={updateNetwork}
                            deleteNetwork={deleteNetwork}
                            addNetwork={addNetwork}
                        />
                    )}
                </div>
            </div>
        </div>
    );
};
