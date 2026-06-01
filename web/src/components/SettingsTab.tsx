
import React, { useState, useEffect } from 'react';
import { Settings, Wifi, User, Zap, Download, Search, Globe, Save, RefreshCw, Plus, Trash2 } from 'lucide-react';
import { useToast } from '../hooks/useToast';
import { AppConfig, NetworkConfig } from '../types';

// Helper component for autojoin channels input that allows typing commas
const AutojoinInput: React.FC<{
    value: string[];
    onChange: (channels: string[]) => void;
    placeholder?: string;
    className?: string;
}> = ({ value, onChange, placeholder, className }) => {
    const [localValue, setLocalValue] = useState(value.join(', '));

    // Sync local value when external value changes (e.g., on initial load)
    useEffect(() => {
        setLocalValue(value.join(', '));
    }, [value.join(',')]); // Only update if the actual array content changes

    const handleBlur = () => {
        // Parse and commit the value on blur
        const channels = localValue
            .split(',')
            .map(c => c.trim())
            .filter(c => c.length > 0);
        onChange(channels);
    };

    return (
        <input
            type="text"
            value={localValue}
            onChange={e => setLocalValue(e.target.value)}
            onBlur={handleBlur}
            placeholder={placeholder}
            className={className}
        />
    );
};

type SettingsSection = 'connection' | 'identity' | 'behavior' | 'dcc' | 'search' | 'networks';

export const SettingsTab: React.FC = () => {
    const [settings, setSettings] = useState<AppConfig | null>(null);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [activeSection, setActiveSection] = useState<SettingsSection>('connection');
    const [newNetworkName, setNewNetworkName] = useState('');
    const { showToast } = useToast();

    useEffect(() => {
        fetchSettings();
    }, []);

    const fetchSettings = async () => {
        try {
            const res = await fetch('/api/settings');
            if (res.ok) {
                const data = await res.json();
                setSettings(data);
            }
        } catch (e) {
            showToast('Failed to load settings', 'error');
        } finally {
            setLoading(false);
        }
    };

    const saveSettings = async () => {
        if (!settings) return;

        setSaving(true);
        try {
            const res = await fetch('/api/settings', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            });
            if (res.ok) {
                showToast('Settings saved!', 'success');
            } else {
                showToast('Failed to save settings', 'error');
            }
        } catch (e) {
            showToast('Failed to save settings', 'error');
        } finally {
            setSaving(false);
        }
    };

    const updateSetting = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
        if (settings) {
            setSettings({ ...settings, [key]: value });
        }
    };

    const addNetwork = async () => {
        if (!newNetworkName.trim() || !settings) return;

        const network: NetworkConfig = {
            host: `irc.${newNetworkName.toLowerCase()}.net`,
            port: 6697,
            ssl: true,
            autojoin_channels: [],
            join_delay_secs: 6
        };

        try {
            await fetch(`/api/settings/networks/${encodeURIComponent(newNetworkName)}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(network)
            });

            setSettings({
                ...settings,
                networks: { ...settings.networks, [newNetworkName]: network }
            });
            setNewNetworkName('');
            showToast('Network added', 'success');
        } catch (e) {
            showToast('Failed to add network', 'error');
        }
    };

    const deleteNetwork = async (name: string) => {
        if (!settings) return;

        try {
            await fetch(`/api/settings/networks/${encodeURIComponent(name)}`, { method: 'DELETE' });
            const newNetworks = { ...settings.networks };
            delete newNetworks[name];
            setSettings({ ...settings, networks: newNetworks });
            showToast('Network deleted', 'success');
        } catch (e) {
            showToast('Failed to delete network', 'error');
        }
    };

    const updateNetwork = (name: string, field: keyof NetworkConfig, value: any) => {
        if (!settings) return;
        setSettings({
            ...settings,
            networks: {
                ...settings.networks,
                [name]: { ...settings.networks[name], [field]: value }
            }
        });
    };

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
                    {activeSection === 'connection' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">Connection Settings</h3>

                            <div className="grid grid-cols-2 gap-6">
                                <label className="flex items-center justify-between">
                                    <span>Enable SSL/TLS</span>
                                    <input
                                        type="checkbox"
                                        checked={settings.use_ssl}
                                        onChange={e => updateSetting('use_ssl', e.target.checked)}
                                        className="w-5 h-5 rounded accent-primary"
                                    />
                                </label>

                                <label className="flex items-center justify-between">
                                    <span>Enable Proxy</span>
                                    <input
                                        type="checkbox"
                                        checked={settings.proxy_enabled}
                                        onChange={e => updateSetting('proxy_enabled', e.target.checked)}
                                        className="w-5 h-5 rounded accent-primary"
                                    />
                                </label>
                            </div>

                            {settings.proxy_enabled && (
                                <div>
                                    <label className="block text-sm text-secondary mb-2">Proxy URL</label>
                                    <input
                                        type="text"
                                        value={settings.proxy_url}
                                        onChange={e => updateSetting('proxy_url', e.target.value)}
                                        placeholder="socks5://127.0.0.1:1080"
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>
                            )}

                            <div className="grid grid-cols-2 gap-6">
                                <div>
                                    <label className="block text-sm text-secondary mb-2">Connect Timeout (seconds)</label>
                                    <input
                                        type="number"
                                        value={settings.connect_timeout}
                                        onChange={e => updateSetting('connect_timeout', parseInt(e.target.value) || 15)}
                                        min={5}
                                        max={60}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm text-secondary mb-2">General Timeout (seconds)</label>
                                    <input
                                        type="number"
                                        value={settings.general_timeout}
                                        onChange={e => updateSetting('general_timeout', parseInt(e.target.value) || 120)}
                                        min={30}
                                        max={300}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {activeSection === 'identity' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">IRC Identity</h3>

                            <div>
                                <label className="block text-sm text-secondary mb-2">Nickname</label>
                                <input
                                    type="text"
                                    value={settings.nickname}
                                    onChange={e => updateSetting('nickname', e.target.value)}
                                    className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                />
                            </div>

                            <div>
                                <label className="block text-sm text-secondary mb-2">Username</label>
                                <input
                                    type="text"
                                    value={settings.username}
                                    onChange={e => updateSetting('username', e.target.value)}
                                    className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                />
                            </div>

                            <div>
                                <label className="block text-sm text-secondary mb-2">Real Name</label>
                                <input
                                    type="text"
                                    value={settings.realname}
                                    onChange={e => updateSetting('realname', e.target.value)}
                                    className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                />
                            </div>
                        </div>
                    )}

                    {activeSection === 'behavior' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">IRC Behavior</h3>

                            <div className="grid grid-cols-2 gap-6">
                                <div>
                                    <label className="block text-sm text-secondary mb-2">Max Retries</label>
                                    <input
                                        type="number"
                                        value={settings.max_retries}
                                        onChange={e => updateSetting('max_retries', parseInt(e.target.value) || 3)}
                                        min={0}
                                        max={10}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm text-secondary mb-2">Retry Delay (seconds)</label>
                                    <input
                                        type="number"
                                        value={settings.retry_delay}
                                        onChange={e => updateSetting('retry_delay', parseInt(e.target.value) || 30)}
                                        min={5}
                                        max={300}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm text-secondary mb-2">Queue Limit (per bot)</label>
                                    <input
                                        type="number"
                                        value={settings.queue_limit}
                                        onChange={e => updateSetting('queue_limit', parseInt(e.target.value) || 2)}
                                        min={1}
                                        max={10}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {activeSection === 'dcc' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">DCC Settings</h3>

                            <div className="grid grid-cols-2 gap-6">
                                <label className="flex items-center justify-between">
                                    <span>Passive DCC Mode</span>
                                    <input
                                        type="checkbox"
                                        checked={settings.passive_dcc}
                                        onChange={e => updateSetting('passive_dcc', e.target.checked)}
                                        className="w-5 h-5 rounded accent-primary"
                                    />
                                </label>

                                <label className="flex items-center justify-between">
                                    <span>Resume Downloads</span>
                                    <input
                                        type="checkbox"
                                        checked={settings.resume_enabled}
                                        onChange={e => updateSetting('resume_enabled', e.target.checked)}
                                        className="w-5 h-5 rounded accent-primary"
                                    />
                                </label>
                            </div>

                            {settings.passive_dcc && (
                                <div className="grid grid-cols-2 gap-6">
                                    <div>
                                        <label className="block text-sm text-secondary mb-2">DCC Port Min</label>
                                        <input
                                            type="number"
                                            value={settings.dcc_port_min}
                                            onChange={e => updateSetting('dcc_port_min', parseInt(e.target.value) || 49152)}
                                            min={1024}
                                            max={65535}
                                            className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                        />
                                    </div>

                                    <div>
                                        <label className="block text-sm text-secondary mb-2">DCC Port Max</label>
                                        <input
                                            type="number"
                                            value={settings.dcc_port_max}
                                            onChange={e => updateSetting('dcc_port_max', parseInt(e.target.value) || 65535)}
                                            min={1024}
                                            max={65535}
                                            className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                        />
                                    </div>
                                </div>
                            )}
                        </div>
                    )}

                    {activeSection === 'search' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">Search Settings</h3>

                            <div>
                                <label className="block text-sm text-secondary mb-3">Enabled Providers</label>
                                <div className="space-y-2">
                                    {['SkullXDCC', 'XDCC.rocks', 'XDCC.eu'].map(provider => (
                                        <label key={provider} className="flex items-center gap-3">
                                            <input
                                                type="checkbox"
                                                checked={settings.enabled_providers.includes(provider)}
                                                onChange={e => {
                                                    const providers = e.target.checked
                                                        ? [...settings.enabled_providers, provider]
                                                        : settings.enabled_providers.filter(p => p !== provider);
                                                    updateSetting('enabled_providers', providers);
                                                }}
                                                className="w-4 h-4 rounded accent-primary"
                                            />
                                            <span>{provider}</span>
                                        </label>
                                    ))}
                                </div>
                            </div>

                            <div className="grid grid-cols-2 gap-6">
                                <div>
                                    <label className="block text-sm text-secondary mb-2">Results Per Page</label>
                                    <input
                                        type="number"
                                        value={settings.results_per_page}
                                        onChange={e => updateSetting('results_per_page', parseInt(e.target.value) || 50)}
                                        min={10}
                                        max={200}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm text-secondary mb-2">Search Timeout (seconds)</label>
                                    <input
                                        type="number"
                                        value={settings.search_timeout}
                                        onChange={e => updateSetting('search_timeout', parseInt(e.target.value) || 30)}
                                        min={10}
                                        max={120}
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {activeSection === 'networks' && (
                        <div className="space-y-6">
                            <h3 className="text-lg font-semibold mb-4">IRC Networks</h3>

                            {/* Add network */}
                            <div className="flex gap-3">
                                <input
                                    type="text"
                                    value={newNetworkName}
                                    onChange={e => setNewNetworkName(e.target.value)}
                                    placeholder="Network name..."
                                    className="flex-1 bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                />
                                <button
                                    onClick={addNetwork}
                                    disabled={!newNetworkName.trim()}
                                    className="flex items-center gap-2 bg-primary hover:bg-primary/80 text-white px-4 py-2 rounded-lg transition-colors disabled:opacity-50"
                                >
                                    <Plus size={18} />
                                    Add
                                </button>
                            </div>

                            {/* Networks list */}
                            <div className="space-y-4">
                                {Object.entries(settings.networks).map(([name, network]) => (
                                    <div key={name} className="bg-surface/50 rounded-lg p-4">
                                        <div className="flex items-center justify-between mb-3">
                                            <span className="font-medium text-primary">{name}</span>
                                            <button
                                                onClick={() => deleteNetwork(name)}
                                                className="text-red-400 hover:text-red-300 p-1"
                                            >
                                                <Trash2 size={16} />
                                            </button>
                                        </div>

                                        <div className="grid grid-cols-3 gap-4">
                                            <div>
                                                <label className="block text-xs text-muted mb-1">Host</label>
                                                <input
                                                    type="text"
                                                    value={network.host}
                                                    onChange={e => updateNetwork(name, 'host', e.target.value)}
                                                    className="w-full bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50"
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-xs text-muted mb-1">Port</label>
                                                <input
                                                    type="number"
                                                    value={network.port}
                                                    onChange={e => updateNetwork(name, 'port', parseInt(e.target.value) || 6697)}
                                                    className="w-full bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50"
                                                />
                                            </div>
                                            <div className="flex items-end pb-1">
                                                <label className="flex items-center gap-2">
                                                    <input
                                                        type="checkbox"
                                                        checked={network.ssl}
                                                        onChange={e => updateNetwork(name, 'ssl', e.target.checked)}
                                                        className="w-4 h-4 rounded accent-primary"
                                                    />
                                                    <span className="text-sm">SSL</span>
                                                </label>
                                            </div>
                                        </div>

                                        <div className="mt-4 grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="block text-xs text-muted mb-1">
                                                    Autojoin Channels (comma separated)
                                                </label>
                                                <AutojoinInput
                                                    value={network.autojoin_channels}
                                                    onChange={(channels) => updateNetwork(name, 'autojoin_channels', channels)}
                                                    placeholder="#chan1, #chan2"
                                                    className="w-full bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50"
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-xs text-muted mb-1">
                                                    Join Delay (seconds)
                                                </label>
                                                <input
                                                    type="number"
                                                    value={network.join_delay_secs}
                                                    onChange={e => updateNetwork(name, 'join_delay_secs', parseInt(e.target.value) || 0)}
                                                    min={0}
                                                    max={300}
                                                    className="w-full bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                ))}

                                {Object.keys(settings.networks).length === 0 && (
                                    <div className="text-center py-8 text-muted">
                                        No networks configured. Add one above.
                                    </div>
                                )}
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};
