import React from 'react';
import { Save } from 'lucide-react';
import { AutodlFilter } from '../../hooks/useAutodl';

interface FilterFormProps {
    form: AutodlFilter;
    setForm: (val: AutodlFilter) => void;
    onSave: () => void;
    onCancel: () => void;
}

export const FilterForm: React.FC<FilterFormProps> = ({ form, setForm, onSave, onCancel }) => {
    return (
        <div className="bg-[#111115] rounded-xl border border-primary/30 p-5 shadow-[0_0_15px_rgba(var(--primary-rgb),0.1)]">
            <div className="flex justify-between items-center mb-4 border-b border-white/10 pb-3">
                <h3 className="font-semibold text-lg text-primary">Edit Filter</h3>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Filter Name</label>
                    <input type="text" value={form.name} onChange={e => setForm({...form, name: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Network</label>
                    <input type="text" value={form.network} onChange={e => setForm({...form, network: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Channels (comma-separated)</label>
                    <input type="text" value={form.channels.join(', ')} onChange={e => setForm({...form, channels: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="#channel1, #channel2" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Bots (comma-separated regex)</label>
                    <input type="text" value={form.bots.join(', ')} onChange={e => setForm({...form, bots: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder=".*" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Lua Match Pattern</label>
                    <input type="text" value={form.match || ''} onChange={e => setForm({...form, match: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="SomeMovie.*1080p" />
                </div>
                <div>
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Lua Exclude Pattern</label>
                    <input type="text" value={form.exclude || ''} onChange={e => setForm({...form, exclude: e.target.value})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="FRENCH|GERMAN" />
                </div>
                <div className="md:col-span-2">
                    <label className="block text-xs uppercase tracking-wider text-secondary mb-1">Smart Keywords (comma-separated)</label>
                    <input type="text" value={(form.keywords || []).join(', ')} onChange={e => setForm({...form, keywords: e.target.value.split(',').map(s=>s.trim()).filter(Boolean)})} className="w-full bg-black/40 border border-white/10 rounded px-3 py-2 text-sm focus:border-primary/50 outline-none" placeholder="event, 2026, world, cup" />
                    <p className="text-xs text-secondary/60 mt-1">Smart keywords are case-insensitive and can appear in any order in the announcement.</p>
                </div>
            </div>

            <div className="flex justify-end gap-3 pt-4 border-t border-white/10">
                <button onClick={onCancel} className="px-4 py-2 text-sm text-secondary hover:text-white transition-colors">Cancel</button>
                <button onClick={onSave} className="flex items-center gap-2 px-6 py-2 bg-primary text-black font-medium rounded-lg hover:bg-primary/90 transition-colors shadow-[0_0_15px_rgba(var(--primary-rgb),0.3)]">
                    <Save size={16} /> Save Filter
                </button>
            </div>
        </div>
    );
};
