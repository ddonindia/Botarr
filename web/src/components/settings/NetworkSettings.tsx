import React, { useState } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { AppConfig, NetworkConfig } from '../../types';

// Helper component for autojoin channels input that allows typing commas
export const AutojoinInput: React.FC<{
    value: string[];
    onChange: (channels: string[]) => void;
    placeholder?: string;
    className?: string;
}> = ({ value, onChange, placeholder, className }) => {
    const [localValue, setLocalValue] = useState(value.join(', '));

    React.useEffect(() => {
        setLocalValue(value.join(', '));
    }, [value.join(',')]);

    const handleBlur = () => {
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

interface NetworkSettingsProps {
    settings: AppConfig;
    updateNetwork: (name: string, field: keyof NetworkConfig, value: any) => void;
    deleteNetwork: (name: string) => void;
    addNetwork: (name: string) => Promise<boolean>;
}

export const NetworkSettings: React.FC<NetworkSettingsProps> = ({
    settings,
    updateNetwork,
    deleteNetwork,
    addNetwork
}) => {
    const [newNetworkName, setNewNetworkName] = useState('');
    const [isAdding, setIsAdding] = useState(false);

    const handleAdd = async () => {
        if (!newNetworkName.trim()) return;
        setIsAdding(true);
        const success = await addNetwork(newNetworkName);
        if (success) {
            setNewNetworkName('');
        }
        setIsAdding(false);
    };

    return (
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
                    onClick={handleAdd}
                    disabled={!newNetworkName.trim() || isAdding}
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
                        <div className="mt-3">
                            <label className="block text-xs text-muted mb-1">
                                NickServ Password <span className="text-muted/60">(optional — leave blank to skip IDENTIFY)</span>
                            </label>
                            <input
                                type="password"
                                value={network.nickserv_password ?? ''}
                                onChange={e => updateNetwork(name, 'nickserv_password', e.target.value)}
                                placeholder="Leave blank if not registered"
                                autoComplete="new-password"
                                className="w-full bg-surface border border-white/10 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-primary/50"
                            />
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
    );
};
