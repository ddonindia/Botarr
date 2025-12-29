import React from 'react';
import { Activity, Download, Server } from 'lucide-react';
import { BotStats } from '../types';
import { formatBytes } from '../utils/format';

interface StatsBarProps {
    stats: BotStats[];
    queueSize: number;
    activeDownloads: number;
}

export const StatsBar: React.FC<StatsBarProps> = ({ stats, queueSize, activeDownloads }) => {
    const totalVolume = stats.reduce((acc, curr) => acc + curr.total_bytes, 0);

    return (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
            <div className="glass-card p-4 flex items-center space-x-4">
                <div className="p-3 bg-primary/20 rounded-full text-primary">
                    <Activity size={24} />
                </div>
                <div>
                    <h3 className="text-secondary text-sm font-medium">Active Downloads</h3>
                    <p className="text-2xl font-bold text-white">{activeDownloads}</p>
                </div>
            </div>

            <div className="glass-card p-4 flex items-center space-x-4">
                <div className="p-3 bg-success/20 rounded-full text-success">
                    <Download size={24} />
                </div>
                <div>
                    <h3 className="text-secondary text-sm font-medium">Total Volume</h3>
                    <p className="text-2xl font-bold text-white">{formatBytes(totalVolume)}</p>
                </div>
            </div>

            <div className="glass-card p-4 flex items-center space-x-4">
                <div className="p-3 bg-warning/20 rounded-full text-warning">
                    <Server size={24} />
                </div>
                <div>
                    <h3 className="text-secondary text-sm font-medium">Queue Size</h3>
                    <p className="text-2xl font-bold text-white">{queueSize}</p>
                </div>
            </div>
        </div>
    );
};
