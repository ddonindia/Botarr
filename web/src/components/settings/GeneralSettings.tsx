import React from 'react';
import { AppConfig } from '../../types';

interface GeneralSettingsProps {
    settings: AppConfig;
    updateSetting: <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => void;
    activeSection: string;
}

export const GeneralSettings: React.FC<GeneralSettingsProps> = ({ settings, updateSetting, activeSection }) => {
    return (
        <>
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

                    <div className="border-t border-white/10 pt-6 mt-6">
                        <h4 className="text-md font-semibold mb-4 text-secondary">Postprocessing</h4>

                        <div className="space-y-4">
                            <label className="flex items-center justify-between">
                                <span>Move completed downloads</span>
                                <input
                                    type="checkbox"
                                    checked={settings.move_completed ?? false}
                                    onChange={e => updateSetting('move_completed', e.target.checked)}
                                    className="w-5 h-5 rounded accent-primary"
                                />
                            </label>

                            {settings.move_completed && (
                                <div>
                                    <label className="block text-sm text-secondary mb-2">Move to directory</label>
                                    <input
                                        type="text"
                                        value={settings.move_completed_dir ?? ''}
                                        onChange={e => updateSetting('move_completed_dir', e.target.value)}
                                        placeholder="/path/to/completed"
                                        className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                    />
                                </div>
                            )}

                            <label className="flex items-center justify-between">
                                <span>Run postprocess script</span>
                                <input
                                    type="checkbox"
                                    checked={settings.postprocess_script_enabled ?? false}
                                    onChange={e => updateSetting('postprocess_script_enabled', e.target.checked)}
                                    className="w-5 h-5 rounded accent-primary"
                                />
                            </label>

                            {settings.postprocess_script_enabled && (
                                <>
                                    <div>
                                        <label className="block text-sm text-secondary mb-2">Script path</label>
                                        <input
                                            type="text"
                                            value={settings.postprocess_script ?? ''}
                                            onChange={e => updateSetting('postprocess_script', e.target.value)}
                                            placeholder="/path/to/script.sh"
                                            className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                        />
                                        <p className="text-xs text-muted mt-1">Script will be called with file path as argument</p>
                                    </div>
                                    <div>
                                        <label className="block text-sm text-secondary mb-2">Script timeout (seconds)</label>
                                        <input
                                            type="number"
                                            value={settings.postprocess_timeout ?? 300}
                                            onChange={e => updateSetting('postprocess_timeout', parseInt(e.target.value) || 300)}
                                            min={10}
                                            max={3600}
                                            className="w-full bg-surface border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary/50"
                                        />
                                    </div>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            )}

            {activeSection === 'search' && (
                <div className="space-y-6">
                    <h3 className="text-lg font-semibold mb-4">Search Settings</h3>

                    <div>
                        <label className="block text-sm text-secondary mb-3">Enabled Providers</label>
                        <div className="space-y-2">
                            {['SkullXDCC', 'XDCC.rocks', 'XDCC.eu', 'NIBL'].map(provider => (
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
        </>
    );
};
