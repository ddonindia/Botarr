import React, { useState } from 'react';
import { X, Download, AlertCircle } from 'lucide-react';
import { useSettings } from '../hooks/useSettings';

interface ManualDownloadModalProps {
    onClose: () => void;
    onSubmit: (url: string) => Promise<void>;
}

export const ManualDownloadModal: React.FC<ManualDownloadModalProps> = ({ onClose, onSubmit }) => {
    const { settings } = useSettings();
    const [input, setInput] = useState('');
    const [network, setNetwork] = useState('');
    const [channel, setChannel] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const availableNetworks = settings ? Object.keys(settings.networks) : [];

    // Automatically select the first network if none is selected and networks are available
    if (!network && availableNetworks.length > 0) {
        setNetwork(availableNetworks[0]);
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        
        let finalUrl = input.trim();
        
        // Check if the input is a standard IRC url
        if (!finalUrl.startsWith('irc://')) {
            // It might be a /msg command, e.g. /msg SunXDCC xdcc send #123
            // Or just 'SunXDCC 123'
            const msgRegex = /^(?:\/msg\s+)?([^\s]+)\s+(?:xdcc\s+send\s+)?#?(\d+)$/i;
            const match = finalUrl.match(msgRegex);
            
            if (match) {
                if (!network) {
                    setError('Please select a network for this bot.');
                    return;
                }
                const botName = match[1];
                const packNum = match[2];
                // channel is optional, default to a generic if empty to satisfy parser
                const finalChannel = channel.trim() || '#xdcc'; 
                finalUrl = `irc://${network}/${finalChannel.replace(/^#/, '')}/${botName}/${packNum}`;
            } else {
                setError('Invalid input format. Use an irc:// link or a /msg command.');
                return;
            }
        }

        setSubmitting(true);
        try {
            await onSubmit(finalUrl);
            onClose();
        } catch (err: any) {
            setError(err.message || 'Failed to submit download');
        } finally {
            setSubmitting(false);
        }
    };

    const isIrcLink = input.trim().startsWith('irc://');

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-surface border border-white/10 rounded-xl shadow-2xl w-full max-w-lg flex flex-col overflow-hidden">
                <div className="flex items-center justify-between p-4 border-b border-white/5 bg-white/5">
                    <div className="flex items-center gap-3">
                        <Download size={20} className="text-primary" />
                        <h3 className="font-semibold text-white">Manual Download</h3>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 rounded-lg hover:bg-white/10 text-secondary hover:text-white transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6 space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-secondary mb-1">
                            XDCC Link or /msg Command
                        </label>
                        <input
                            type="text"
                            value={input}
                            onChange={(e) => setInput(e.target.value)}
                            placeholder="irc://... or /msg BotName xdcc send #123"
                            className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm text-white focus:border-primary/50 outline-none placeholder-muted"
                            autoFocus
                        />
                        <p className="text-xs text-muted mt-1">
                            Paste a full <code>irc://</code> link, or a standard <code>/msg</code> command.
                        </p>
                    </div>

                    {!isIrcLink && input.trim() !== '' && (
                        <div className="p-4 rounded-lg bg-primary/5 border border-primary/20 space-y-3 animate-in slide-in-from-top-2">
                            <p className="text-sm text-primary flex items-center gap-2">
                                <AlertCircle size={16} />
                                Please specify the network for this bot
                            </p>
                            
                            <div>
                                <label className="block text-xs font-medium text-secondary mb-1">Network</label>
                                <select 
                                    value={network} 
                                    onChange={(e) => setNetwork(e.target.value)}
                                    className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm text-white focus:border-primary/50 outline-none"
                                >
                                    {availableNetworks.map(n => (
                                        <option key={n} value={n}>{n}</option>
                                    ))}
                                </select>
                            </div>
                            
                            <div>
                                <label className="block text-xs font-medium text-secondary mb-1">Channel (Optional)</label>
                                <input
                                    type="text"
                                    value={channel}
                                    onChange={(e) => setChannel(e.target.value)}
                                    placeholder="#channel"
                                    className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm text-white focus:border-primary/50 outline-none placeholder-muted"
                                />
                            </div>
                        </div>
                    )}

                    {error && (
                        <div className="text-sm text-red-400 p-3 rounded bg-red-400/10 border border-red-400/20">
                            {error}
                        </div>
                    )}

                    <div className="flex justify-end gap-3 pt-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="btn btn-secondary"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={!input.trim() || submitting}
                            className="btn btn-primary"
                        >
                            {submitting ? 'Starting...' : 'Download'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};
