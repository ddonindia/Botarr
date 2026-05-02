import React, { useEffect, useState, useRef } from 'react';
import { Terminal, Activity, FileCode2 } from 'lucide-react';

interface MonitorStatus {
    plugin: string;
    network: string;
    channel: string;
    status: string;
}

interface PluginStatusResponse {
    loaded_scripts: string[];
    logs: Record<string, string[]>;
    active_monitors: MonitorStatus[];
    raw_irc_logs: string[];
}

export const PluginsTab: React.FC = () => {
    const [status, setStatus] = useState<PluginStatusResponse>({
        loaded_scripts: [],
        logs: {},
        active_monitors: [],
        raw_irc_logs: [],
    });
    const [activePlugin, setActivePlugin] = useState<string>("System");
    const [isLoading, setIsLoading] = useState(true);
    const logsEndRef = useRef<HTMLDivElement>(null);
    const rawLogsEndRef = useRef<HTMLDivElement>(null);

    const fetchStatus = async () => {
        try {
            const res = await fetch('/api/plugins/status');
            const data = await res.json();
            setStatus(data);
            setIsLoading(false);
        } catch (e) {
            console.error("Failed to fetch plugin status", e);
        }
    };

    useEffect(() => {
        fetchStatus();
        const interval = setInterval(fetchStatus, 2000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        // Auto scroll to bottom of logs
        if (logsEndRef.current) {
            logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
        if (rawLogsEndRef.current) {
            rawLogsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [status.logs, status.raw_irc_logs]);

    if (isLoading) {
        return <div className="flex items-center justify-center h-64"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div></div>;
    }

    const availableTabs = ["System", ...status.loaded_scripts];
    const currentLogs = activePlugin === "System" ? status.raw_irc_logs : (status.logs[activePlugin] || []);
    const currentMonitors = status.active_monitors.filter(m => activePlugin === "System" || m.plugin === activePlugin);

    return (
        <div className="flex flex-col gap-6 animate-fade-in pb-8">
            <div className="text-center mb-4">
                <h1 className="text-3xl font-bold mb-2">Plugin System</h1>
                <p className="text-secondary">Monitor your loaded Lua scripts and background connections</p>
            </div>

            <div className="flex flex-col md:flex-row gap-6 items-stretch min-h-[600px]">
                {/* Left Sidebar (Tabs) */}
                <div className="w-full md:w-64 glass p-4 rounded-2xl border border-white/5 shadow-xl shadow-black/20 flex flex-col shrink-0">
                    <h2 className="text-lg font-semibold text-white mb-4 px-2">Plugins</h2>
                    <div className="flex flex-col gap-2">
                        {availableTabs.map((tab) => (
                            <button
                                key={tab}
                                onClick={() => setActivePlugin(tab)}
                                className={`px-4 py-3 rounded-xl text-left font-medium transition-all flex items-center gap-3 ${
                                    activePlugin === tab 
                                    ? 'bg-primary/20 text-primary border border-primary/30 shadow-[0_0_15px_rgba(var(--primary-rgb),0.15)]' 
                                    : 'text-secondary hover:bg-white/5 hover:text-white border border-transparent'
                                }`}
                            >
                                {tab === "System" ? <Activity size={18} /> : <FileCode2 size={18} />}
                                {tab}
                                {tab !== "System" && (
                                    <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)] ml-auto"></div>
                                )}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Right Content Area */}
                <div className="flex-1 flex flex-col gap-6">
                    
                    {/* Active Monitors Card */}
                    <div className="glass p-6 rounded-2xl border border-white/5 shadow-xl shadow-black/20 flex flex-col">
                        <div className="flex items-center gap-3 mb-6 pb-4 border-b border-white/5">
                            <div className="p-2 rounded-lg bg-blue-500/20 text-blue-400">
                                <Activity size={24} />
                            </div>
                            <div>
                                <h2 className="text-xl font-semibold text-white">
                                    {activePlugin === "System" ? "All Active Monitors" : `${activePlugin} Monitors`}
                                </h2>
                                <p className="text-sm text-secondary">Persistent IRC connections managed by this plugin</p>
                            </div>
                        </div>
                        
                        <div className="flex-1">
                            {currentMonitors.length === 0 ? (
                                <div className="text-center text-secondary py-4 italic">No active channel monitors running for {activePlugin}</div>
                            ) : (
                                <ul className="space-y-3">
                                    {currentMonitors.map((mon, idx) => (
                                        <li key={idx} className="flex items-center justify-between p-3 rounded-lg bg-black/20 border border-white/5">
                                            <div className="flex flex-col">
                                                <span className="font-medium text-white">{mon.channel}</span>
                                                <span className="text-xs text-secondary">{mon.network} {activePlugin === "System" ? `(${mon.plugin})` : ''}</span>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                {mon.status === 'Connected' ? (
                                                    <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse"></div>
                                                ) : (
                                                    <div className="w-2 h-2 rounded-full bg-yellow-500 shadow-[0_0_8px_rgba(234,179,8,0.6)]"></div>
                                                )}
                                                <span className="text-sm text-secondary">{mon.status}</span>
                                            </div>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </div>
                    </div>

                    {/* Console Output */}
                    <div className="glass rounded-2xl border border-white/5 shadow-xl shadow-black/20 flex flex-col overflow-hidden flex-1 min-h-[300px]">
                        <div className="flex items-center gap-3 p-4 bg-black/40 border-b border-white/5">
                            <Terminal size={20} className={activePlugin === "System" ? "text-blue-400" : "text-primary"} />
                            <h2 className="text-lg font-semibold text-white">
                                {activePlugin === "System" ? "Raw IRC Feed" : `${activePlugin} Console`}
                            </h2>
                        </div>
                        <div className="bg-[#0a0a0c] p-4 flex-1 overflow-y-auto font-mono text-sm max-h-[400px]">
                            {currentLogs.length === 0 ? (
                                <div className="text-secondary italic">Waiting for logs...</div>
                            ) : (
                                <div className="space-y-1">
                                    {currentLogs.map((log, idx) => (
                                        <div key={idx} className={activePlugin === "System" ? "text-gray-400 whitespace-pre-wrap break-all text-xs" : "text-gray-300"}>
                                            {log}
                                        </div>
                                    ))}
                                    <div ref={activePlugin === "System" ? rawLogsEndRef : logsEndRef} />
                                </div>
                            )}
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
};
